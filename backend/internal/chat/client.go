package chat

import (
	"database/sql"
	"encoding/json"
	"log/slog"
	"net/http"
	"sync"
	"time"

	"github.com/gorilla/websocket"
	"github.com/umar/day22-chat/internal/auth"
	"github.com/umar/day22-chat/internal/database"
)

const (
	writeWait      = 10 * time.Second
	pongWait       = 60 * time.Second
	pingPeriod     = 54 * time.Second
	maxMessageSize = 4096
)

var upgrader = websocket.Upgrader{
	ReadBufferSize:  1024,
	WriteBufferSize: 1024,
	CheckOrigin:     func(r *http.Request) bool { return true },
}

type Client struct {
	hub      *Hub
	conn     *websocket.Conn
	UserID   string
	Username string
	rooms    map[string]bool
	send     chan []byte
	mu       sync.Mutex
}

func ServeWS(hub *Hub, db *sql.DB, jwtSecret string) http.HandlerFunc {
	return func(w http.ResponseWriter, r *http.Request) {
		token := r.URL.Query().Get("token")
		if token == "" {
			http.Error(w, "missing token", http.StatusUnauthorized)
			return
		}

		claims, err := auth.ValidateToken(token, jwtSecret)
		if err != nil {
			http.Error(w, "invalid token", http.StatusUnauthorized)
			return
		}

		conn, err := upgrader.Upgrade(w, r, nil)
		if err != nil {
			slog.Error("websocket upgrade failed", "error", err)
			return
		}

		rooms := make(map[string]bool)
		userRooms, err := database.GetRoomsForUser(db, claims.UserID)
		if err == nil {
			for _, room := range userRooms {
				rooms[room.ID] = true
			}
		}

		client := &Client{
			hub:      hub,
			conn:     conn,
			UserID:   claims.UserID,
			Username: claims.Username,
			rooms:    rooms,
			send:     make(chan []byte, 256),
		}

		hub.register <- client
		go client.writePump()
		go client.readPump()
	}
}

func (c *Client) readPump() {
	defer func() {
		c.hub.unregister <- c
		c.conn.Close()
	}()

	c.conn.SetReadLimit(maxMessageSize)
	c.conn.SetReadDeadline(time.Now().Add(pongWait))
	c.conn.SetPongHandler(func(string) error {
		c.conn.SetReadDeadline(time.Now().Add(pongWait))
		return nil
	})

	for {
		_, message, err := c.conn.ReadMessage()
		if err != nil {
			if websocket.IsUnexpectedCloseError(err, websocket.CloseGoingAway, websocket.CloseNormalClosure) {
				slog.Error("ws read error", "error", err, "user_id", c.UserID)
			}
			break
		}

		var msg WSMessage
		if err := json.Unmarshal(message, &msg); err != nil {
			continue
		}

		c.handleMessage(msg)
	}
}

func (c *Client) writePump() {
	ticker := time.NewTicker(pingPeriod)
	defer func() {
		ticker.Stop()
		c.conn.Close()
	}()

	for {
		select {
		case message, ok := <-c.send:
			c.conn.SetWriteDeadline(time.Now().Add(writeWait))
			if !ok {
				c.conn.WriteMessage(websocket.CloseMessage, []byte{})
				return
			}
			if err := c.conn.WriteMessage(websocket.TextMessage, message); err != nil {
				return
			}
		case <-ticker.C:
			c.conn.SetWriteDeadline(time.Now().Add(writeWait))
			if err := c.conn.WriteMessage(websocket.PingMessage, nil); err != nil {
				return
			}
		}
	}
}

func (c *Client) handleMessage(msg WSMessage) {
	switch msg.Type {
	case TypeMessageSend:
		var payload SendMessagePayload
		if err := json.Unmarshal(msg.Payload, &payload); err != nil {
			return
		}
		HandleSendMessage(c, payload)
	case TypeRoomJoin:
		var payload RoomPayload
		if err := json.Unmarshal(msg.Payload, &payload); err != nil {
			return
		}
		HandleRoomJoin(c, payload)
	case TypeRoomLeave:
		var payload RoomPayload
		if err := json.Unmarshal(msg.Payload, &payload); err != nil {
			return
		}
		HandleRoomLeave(c, payload)
	case TypeTypingStart:
		var payload RoomPayload
		if err := json.Unmarshal(msg.Payload, &payload); err != nil {
			return
		}
		HandleTyping(c, payload, true)
	case TypeTypingStop:
		var payload RoomPayload
		if err := json.Unmarshal(msg.Payload, &payload); err != nil {
			return
		}
		HandleTyping(c, payload, false)
	case TypeMessageRead:
		var payload ReadPayload
		if err := json.Unmarshal(msg.Payload, &payload); err != nil {
			return
		}
		HandleMessageRead(c, payload)
	case TypePing:
		data, _ := NewWSMessage(TypePong, nil)
		select {
		case c.send <- data:
		default:
		}
	}
}
