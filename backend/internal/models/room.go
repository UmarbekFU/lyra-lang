package models

import "time"

type Room struct {
	ID        string    `json:"id"`
	Name      string    `json:"name"`
	Type      string    `json:"type"`
	CreatedBy string    `json:"created_by"`
	CreatedAt time.Time `json:"created_at"`
}

type RoomWithUnread struct {
	Room
	UnreadCount   int        `json:"unread_count"`
	LastMessage   string     `json:"last_message"`
	LastMessageAt *time.Time `json:"last_message_at,omitempty"`
}
