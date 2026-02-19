package chat

import "encoding/json"

const (
	TypeMessageSend  = "message.send"
	TypeRoomJoin     = "room.join"
	TypeRoomLeave    = "room.leave"
	TypeTypingStart  = "typing.start"
	TypeTypingStop   = "typing.stop"
	TypeMessageRead  = "message.read"
	TypePing         = "ping"

	TypeMessageNew       = "message.new"
	TypeTypingUpdate     = "typing.update"
	TypePresenceUpdate   = "presence.update"
	TypeReadReceipt      = "read_receipt.update"
	TypeUnreadUpdate     = "unread.update"
	TypeRoomMemberJoined = "room.member_joined"
	TypeRoomMemberLeft   = "room.member_left"
	TypeError            = "error"
	TypePong             = "pong"
)

type WSMessage struct {
	Type    string          `json:"type"`
	Payload json.RawMessage `json:"payload,omitempty"`
}

type SendMessagePayload struct {
	RoomID  string `json:"room_id"`
	Content string `json:"content"`
}

type RoomPayload struct {
	RoomID string `json:"room_id"`
}

type ReadPayload struct {
	RoomID    string `json:"room_id"`
	Timestamp string `json:"timestamp"`
}

type NewMessagePayload struct {
	ID             string `json:"id"`
	RoomID         string `json:"room_id"`
	SenderID       string `json:"sender_id"`
	SenderUsername  string `json:"sender_username"`
	Content        string `json:"content"`
	CreatedAt      string `json:"created_at"`
}

type TypingUpdatePayload struct {
	RoomID   string `json:"room_id"`
	UserID   string `json:"user_id"`
	Username string `json:"username"`
	IsTyping bool   `json:"is_typing"`
}

type PresenceUpdatePayload struct {
	UserID   string `json:"user_id"`
	Username string `json:"username"`
	Status   string `json:"status"`
}

type ReadReceiptPayload struct {
	RoomID     string `json:"room_id"`
	UserID     string `json:"user_id"`
	Username   string `json:"username"`
	LastReadAt string `json:"last_read_at"`
}

type UnreadUpdatePayload struct {
	RoomID string `json:"room_id"`
	Count  int    `json:"count"`
}

type MemberEventPayload struct {
	RoomID   string `json:"room_id"`
	UserID   string `json:"user_id"`
	Username string `json:"username"`
}

type ErrorPayload struct {
	Message string `json:"message"`
	Code    string `json:"code"`
}

func NewWSMessage(msgType string, payload interface{}) ([]byte, error) {
	var p json.RawMessage
	if payload != nil {
		var err error
		p, err = json.Marshal(payload)
		if err != nil {
			return nil, err
		}
	}
	msg := WSMessage{Type: msgType, Payload: p}
	return json.Marshal(msg)
}
