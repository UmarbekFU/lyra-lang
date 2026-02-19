package chat

import (
	"log/slog"
	"time"

	"github.com/umar/day22-chat/internal/database"
)

func HandleSendMessage(c *Client, payload SendMessagePayload) {
	if payload.Content == "" || payload.RoomID == "" {
		sendError(c, "content and room_id are required", "INVALID_PAYLOAD")
		return
	}

	isMember, err := database.IsRoomMember(c.hub.DB, payload.RoomID, c.UserID)
	if err != nil || !isMember {
		sendError(c, "not a member of this room", "NOT_MEMBER")
		return
	}

	msg, err := database.CreateMessage(c.hub.DB, payload.RoomID, c.UserID, payload.Content)
	if err != nil {
		slog.Error("failed to create message", "error", err)
		sendError(c, "failed to send message", "INTERNAL_ERROR")
		return
	}

	data, err := NewWSMessage(TypeMessageNew, NewMessagePayload{
		ID:             msg.ID,
		RoomID:         msg.RoomID,
		SenderID:       msg.SenderID,
		SenderUsername:  msg.SenderUsername,
		Content:        msg.Content,
		CreatedAt:      msg.CreatedAt.Format(time.RFC3339),
	})
	if err != nil {
		return
	}

	c.hub.BroadcastToRoom(payload.RoomID, data, "")

	go sendUnreadUpdates(c.hub, payload.RoomID, c.UserID)
}

func HandleRoomJoin(c *Client, payload RoomPayload) {
	if err := database.AddRoomMember(c.hub.DB, payload.RoomID, c.UserID); err != nil {
		slog.Error("failed to join room", "error", err)
		return
	}

	c.mu.Lock()
	c.rooms[payload.RoomID] = true
	c.mu.Unlock()

	data, _ := NewWSMessage(TypeRoomMemberJoined, MemberEventPayload{
		RoomID:   payload.RoomID,
		UserID:   c.UserID,
		Username: c.Username,
	})
	c.hub.BroadcastToRoom(payload.RoomID, data, "")
}

func HandleRoomLeave(c *Client, payload RoomPayload) {
	if err := database.RemoveRoomMember(c.hub.DB, payload.RoomID, c.UserID); err != nil {
		return
	}

	c.mu.Lock()
	delete(c.rooms, payload.RoomID)
	c.mu.Unlock()

	data, _ := NewWSMessage(TypeRoomMemberLeft, MemberEventPayload{
		RoomID:   payload.RoomID,
		UserID:   c.UserID,
		Username: c.Username,
	})
	c.hub.BroadcastToRoom(payload.RoomID, data, "")
}

func HandleTyping(c *Client, payload RoomPayload, isTyping bool) {
	data, _ := NewWSMessage(TypeTypingUpdate, TypingUpdatePayload{
		RoomID:   payload.RoomID,
		UserID:   c.UserID,
		Username: c.Username,
		IsTyping: isTyping,
	})
	c.hub.BroadcastToRoom(payload.RoomID, data, c.UserID)
}

func HandleMessageRead(c *Client, payload ReadPayload) {
	t, err := time.Parse(time.RFC3339, payload.Timestamp)
	if err != nil {
		return
	}

	if err := database.UpdateLastRead(c.hub.DB, payload.RoomID, c.UserID, t); err != nil {
		return
	}

	data, _ := NewWSMessage(TypeReadReceipt, ReadReceiptPayload{
		RoomID:     payload.RoomID,
		UserID:     c.UserID,
		Username:   c.Username,
		LastReadAt: payload.Timestamp,
	})
	c.hub.BroadcastToRoom(payload.RoomID, data, c.UserID)

	unreadData, _ := NewWSMessage(TypeUnreadUpdate, UnreadUpdatePayload{
		RoomID: payload.RoomID,
		Count:  0,
	})
	c.hub.SendToUser(c.UserID, unreadData)
}

func sendUnreadUpdates(hub *Hub, roomID, senderID string) {
	members, err := database.GetRoomMembers(hub.DB, roomID)
	if err != nil {
		return
	}
	for _, member := range members {
		if member.ID == senderID {
			continue
		}
		count, err := database.GetUnreadCount(hub.DB, roomID, member.ID)
		if err != nil {
			continue
		}
		data, _ := NewWSMessage(TypeUnreadUpdate, UnreadUpdatePayload{
			RoomID: roomID,
			Count:  count,
		})
		hub.SendToUser(member.ID, data)
	}
}

func sendError(c *Client, message, code string) {
	data, _ := NewWSMessage(TypeError, ErrorPayload{
		Message: message,
		Code:    code,
	})
	select {
	case c.send <- data:
	default:
	}
}
