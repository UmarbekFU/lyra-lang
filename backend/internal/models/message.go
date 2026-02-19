package models

import "time"

type Message struct {
	ID        string    `json:"id"`
	RoomID    string    `json:"room_id"`
	SenderID  string    `json:"sender_id"`
	Content   string    `json:"content"`
	CreatedAt time.Time `json:"created_at"`
}

type MessageWithSender struct {
	Message
	SenderUsername string `json:"sender_username"`
}
