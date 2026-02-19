package chat

import (
	"database/sql"
	"log/slog"
	"sync"

	"github.com/redis/go-redis/v9"
)

type BroadcastMessage struct {
	RoomID        string
	Data          []byte
	ExcludeUserID string
}

type Hub struct {
	clients    map[string]*Client
	mu         sync.RWMutex

	register   chan *Client
	unregister chan *Client
	broadcast  chan *BroadcastMessage

	DB    *sql.DB
	Redis *redis.Client
}

func NewHub(db *sql.DB, redisClient *redis.Client) *Hub {
	return &Hub{
		clients:    make(map[string]*Client),
		register:   make(chan *Client),
		unregister: make(chan *Client),
		broadcast:  make(chan *BroadcastMessage, 256),
		DB:         db,
		Redis:      redisClient,
	}
}

func (h *Hub) Run() {
	for {
		select {
		case client := <-h.register:
			h.mu.Lock()
			if old, ok := h.clients[client.UserID]; ok {
				close(old.send)
			}
			h.clients[client.UserID] = client
			h.mu.Unlock()
			slog.Info("client connected", "user_id", client.UserID, "username", client.Username)
			h.broadcastPresence(client.UserID, client.Username, "online")

		case client := <-h.unregister:
			h.mu.Lock()
			if existing, ok := h.clients[client.UserID]; ok && existing == client {
				delete(h.clients, client.UserID)
				close(client.send)
			}
			h.mu.Unlock()
			slog.Info("client disconnected", "user_id", client.UserID)
			h.broadcastPresence(client.UserID, client.Username, "offline")

		case msg := <-h.broadcast:
			h.mu.RLock()
			for userID, client := range h.clients {
				if userID == msg.ExcludeUserID {
					continue
				}
				if _, ok := client.rooms[msg.RoomID]; ok {
					select {
					case client.send <- msg.Data:
					default:
						close(client.send)
						delete(h.clients, userID)
					}
				}
			}
			h.mu.RUnlock()
		}
	}
}

func (h *Hub) broadcastPresence(userID, username, status string) {
	data, err := NewWSMessage(TypePresenceUpdate, PresenceUpdatePayload{
		UserID:   userID,
		Username: username,
		Status:   status,
	})
	if err != nil {
		return
	}
	h.mu.RLock()
	for _, client := range h.clients {
		select {
		case client.send <- data:
		default:
		}
	}
	h.mu.RUnlock()
}

func (h *Hub) BroadcastToRoom(roomID string, data []byte, excludeUserID string) {
	h.broadcast <- &BroadcastMessage{
		RoomID:        roomID,
		Data:          data,
		ExcludeUserID: excludeUserID,
	}
}

func (h *Hub) SendToUser(userID string, data []byte) {
	h.mu.RLock()
	client, ok := h.clients[userID]
	h.mu.RUnlock()
	if ok {
		select {
		case client.send <- data:
		default:
		}
	}
}

func (h *Hub) GetOnlineUserIDs() []string {
	h.mu.RLock()
	defer h.mu.RUnlock()
	ids := make([]string, 0, len(h.clients))
	for id := range h.clients {
		ids = append(ids, id)
	}
	return ids
}

func (h *Hub) Shutdown() {
	h.mu.Lock()
	defer h.mu.Unlock()
	for _, client := range h.clients {
		close(client.send)
	}
}
