package models

type PresenceStatus struct {
	UserID   string `json:"user_id"`
	Username string `json:"username"`
	Status   string `json:"status"`
}
