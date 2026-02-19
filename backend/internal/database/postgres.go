package database

import (
	"database/sql"
	"fmt"
	"time"

	"github.com/umar/day22-chat/internal/models"
	_ "github.com/lib/pq"
)

func InitDB(databaseURL string) (*sql.DB, error) {
	db, err := sql.Open("postgres", databaseURL)
	if err != nil {
		return nil, fmt.Errorf("failed to open database: %w", err)
	}
	db.SetMaxOpenConns(25)
	db.SetMaxIdleConns(5)
	db.SetConnMaxLifetime(5 * time.Minute)
	if err := db.Ping(); err != nil {
		return nil, fmt.Errorf("failed to ping database: %w", err)
	}
	return db, nil
}

// --- Users ---

func CreateUser(db *sql.DB, username, email, passwordHash string) (*models.User, error) {
	var u models.User
	err := db.QueryRow(
		`INSERT INTO users (username, email, password) VALUES ($1, $2, $3)
		 RETURNING id, username, email, avatar_url, created_at`,
		username, email, passwordHash,
	).Scan(&u.ID, &u.Username, &u.Email, &u.AvatarURL, &u.CreatedAt)
	if err != nil {
		return nil, fmt.Errorf("failed to create user: %w", err)
	}
	return &u, nil
}

func GetUserByUsername(db *sql.DB, username string) (*models.User, error) {
	var u models.User
	err := db.QueryRow(
		`SELECT id, username, email, password, avatar_url, created_at FROM users WHERE username = $1`,
		username,
	).Scan(&u.ID, &u.Username, &u.Email, &u.Password, &u.AvatarURL, &u.CreatedAt)
	if err != nil {
		if err == sql.ErrNoRows {
			return nil, nil
		}
		return nil, fmt.Errorf("failed to get user: %w", err)
	}
	return &u, nil
}

func GetUserByID(db *sql.DB, id string) (*models.User, error) {
	var u models.User
	err := db.QueryRow(
		`SELECT id, username, email, avatar_url, created_at FROM users WHERE id = $1`,
		id,
	).Scan(&u.ID, &u.Username, &u.Email, &u.AvatarURL, &u.CreatedAt)
	if err != nil {
		if err == sql.ErrNoRows {
			return nil, nil
		}
		return nil, fmt.Errorf("failed to get user: %w", err)
	}
	return &u, nil
}

func SearchUsers(db *sql.DB, query string, limit int) ([]models.User, error) {
	rows, err := db.Query(
		`SELECT id, username, email, avatar_url, created_at FROM users
		 WHERE username ILIKE $1 LIMIT $2`,
		"%"+query+"%", limit,
	)
	if err != nil {
		return nil, fmt.Errorf("failed to search users: %w", err)
	}
	defer rows.Close()

	var users []models.User
	for rows.Next() {
		var u models.User
		if err := rows.Scan(&u.ID, &u.Username, &u.Email, &u.AvatarURL, &u.CreatedAt); err != nil {
			return nil, err
		}
		users = append(users, u)
	}
	if users == nil {
		users = []models.User{}
	}
	return users, nil
}

// --- Rooms ---

func CreateRoom(db *sql.DB, name, roomType, createdBy string) (*models.Room, error) {
	var r models.Room
	err := db.QueryRow(
		`INSERT INTO rooms (name, type, created_by) VALUES ($1, $2, $3)
		 RETURNING id, name, type, created_by, created_at`,
		name, roomType, createdBy,
	).Scan(&r.ID, &r.Name, &r.Type, &r.CreatedBy, &r.CreatedAt)
	if err != nil {
		return nil, fmt.Errorf("failed to create room: %w", err)
	}
	return &r, nil
}

func GetRoomByID(db *sql.DB, id string) (*models.Room, error) {
	var r models.Room
	err := db.QueryRow(
		`SELECT id, name, type, COALESCE(created_by::text, ''), created_at FROM rooms WHERE id = $1`,
		id,
	).Scan(&r.ID, &r.Name, &r.Type, &r.CreatedBy, &r.CreatedAt)
	if err != nil {
		if err == sql.ErrNoRows {
			return nil, nil
		}
		return nil, fmt.Errorf("failed to get room: %w", err)
	}
	return &r, nil
}

func GetRoomsForUser(db *sql.DB, userID string) ([]models.RoomWithUnread, error) {
	rows, err := db.Query(`
		SELECT r.id, r.name, r.type, COALESCE(r.created_by::text, ''), r.created_at,
		       COALESCE(unread.cnt, 0),
		       COALESCE(last_msg.content, ''),
		       last_msg.created_at
		FROM rooms r
		JOIN room_members rm ON r.id = rm.room_id
		LEFT JOIN LATERAL (
		    SELECT COUNT(*) AS cnt FROM messages m
		    WHERE m.room_id = r.id AND m.created_at > rm.last_read_at AND m.sender_id != $1
		) unread ON true
		LEFT JOIN LATERAL (
		    SELECT content, created_at FROM messages
		    WHERE room_id = r.id ORDER BY created_at DESC LIMIT 1
		) last_msg ON true
		WHERE rm.user_id = $1
		ORDER BY COALESCE(last_msg.created_at, r.created_at) DESC
	`, userID)
	if err != nil {
		return nil, fmt.Errorf("failed to get rooms: %w", err)
	}
	defer rows.Close()

	var rooms []models.RoomWithUnread
	for rows.Next() {
		var r models.RoomWithUnread
		var lastMsgAt *time.Time
		if err := rows.Scan(&r.ID, &r.Name, &r.Type, &r.CreatedBy, &r.CreatedAt,
			&r.UnreadCount, &r.LastMessage, &lastMsgAt); err != nil {
			return nil, err
		}
		r.LastMessageAt = lastMsgAt
		rooms = append(rooms, r)
	}
	if rooms == nil {
		rooms = []models.RoomWithUnread{}
	}
	return rooms, nil
}

func GetDMRooms(db *sql.DB, userID string) ([]models.RoomWithUnread, error) {
	rows, err := db.Query(`
		SELECT r.id, r.name, r.type, COALESCE(r.created_by::text, ''), r.created_at,
		       COALESCE(unread.cnt, 0),
		       COALESCE(last_msg.content, ''),
		       last_msg.created_at
		FROM rooms r
		JOIN room_members rm ON r.id = rm.room_id
		LEFT JOIN LATERAL (
		    SELECT COUNT(*) AS cnt FROM messages m
		    WHERE m.room_id = r.id AND m.created_at > rm.last_read_at AND m.sender_id != $1
		) unread ON true
		LEFT JOIN LATERAL (
		    SELECT content, created_at FROM messages
		    WHERE room_id = r.id ORDER BY created_at DESC LIMIT 1
		) last_msg ON true
		WHERE rm.user_id = $1 AND r.type = 'dm'
		ORDER BY COALESCE(last_msg.created_at, r.created_at) DESC
	`, userID)
	if err != nil {
		return nil, fmt.Errorf("failed to get DM rooms: %w", err)
	}
	defer rows.Close()

	var rooms []models.RoomWithUnread
	for rows.Next() {
		var r models.RoomWithUnread
		var lastMsgAt *time.Time
		if err := rows.Scan(&r.ID, &r.Name, &r.Type, &r.CreatedBy, &r.CreatedAt,
			&r.UnreadCount, &r.LastMessage, &lastMsgAt); err != nil {
			return nil, err
		}
		r.LastMessageAt = lastMsgAt
		rooms = append(rooms, r)
	}
	if rooms == nil {
		rooms = []models.RoomWithUnread{}
	}
	return rooms, nil
}

func GetOrCreateDMRoom(db *sql.DB, userID1, userID2 string) (*models.Room, error) {
	var roomID string
	err := db.QueryRow(`
		SELECT rm1.room_id FROM room_members rm1
		JOIN room_members rm2 ON rm1.room_id = rm2.room_id
		JOIN rooms r ON r.id = rm1.room_id
		WHERE rm1.user_id = $1 AND rm2.user_id = $2 AND r.type = 'dm'
		LIMIT 1
	`, userID1, userID2).Scan(&roomID)

	if err == nil {
		return GetRoomByID(db, roomID)
	}
	if err != sql.ErrNoRows {
		return nil, fmt.Errorf("failed to check existing DM: %w", err)
	}

	room, err := CreateRoom(db, "", "dm", userID1)
	if err != nil {
		return nil, err
	}
	if err := AddRoomMember(db, room.ID, userID1); err != nil {
		return nil, err
	}
	if err := AddRoomMember(db, room.ID, userID2); err != nil {
		return nil, err
	}
	return room, nil
}

func AddRoomMember(db *sql.DB, roomID, userID string) error {
	_, err := db.Exec(
		`INSERT INTO room_members (room_id, user_id) VALUES ($1, $2) ON CONFLICT DO NOTHING`,
		roomID, userID,
	)
	return err
}

func RemoveRoomMember(db *sql.DB, roomID, userID string) error {
	_, err := db.Exec(`DELETE FROM room_members WHERE room_id = $1 AND user_id = $2`, roomID, userID)
	return err
}

func GetRoomMembers(db *sql.DB, roomID string) ([]models.User, error) {
	rows, err := db.Query(`
		SELECT u.id, u.username, u.email, u.avatar_url, u.created_at
		FROM users u JOIN room_members rm ON u.id = rm.user_id
		WHERE rm.room_id = $1
	`, roomID)
	if err != nil {
		return nil, fmt.Errorf("failed to get room members: %w", err)
	}
	defer rows.Close()

	var users []models.User
	for rows.Next() {
		var u models.User
		if err := rows.Scan(&u.ID, &u.Username, &u.Email, &u.AvatarURL, &u.CreatedAt); err != nil {
			return nil, err
		}
		users = append(users, u)
	}
	if users == nil {
		users = []models.User{}
	}
	return users, nil
}

func IsRoomMember(db *sql.DB, roomID, userID string) (bool, error) {
	var exists bool
	err := db.QueryRow(
		`SELECT EXISTS(SELECT 1 FROM room_members WHERE room_id = $1 AND user_id = $2)`,
		roomID, userID,
	).Scan(&exists)
	return exists, err
}

// --- Messages ---

func CreateMessage(db *sql.DB, roomID, senderID, content string) (*models.MessageWithSender, error) {
	var m models.MessageWithSender
	err := db.QueryRow(`
		WITH inserted AS (
		    INSERT INTO messages (room_id, sender_id, content)
		    VALUES ($1, $2, $3)
		    RETURNING id, room_id, sender_id, content, created_at
		)
		SELECT i.id, i.room_id, i.sender_id, i.content, i.created_at, u.username
		FROM inserted i JOIN users u ON i.sender_id = u.id
	`, roomID, senderID, content,
	).Scan(&m.ID, &m.RoomID, &m.SenderID, &m.Content, &m.CreatedAt, &m.SenderUsername)
	if err != nil {
		return nil, fmt.Errorf("failed to create message: %w", err)
	}
	return &m, nil
}

func GetMessages(db *sql.DB, roomID string, before time.Time, limit int) ([]models.MessageWithSender, error) {
	rows, err := db.Query(`
		SELECT m.id, m.room_id, m.sender_id, m.content, m.created_at, u.username
		FROM messages m JOIN users u ON m.sender_id = u.id
		WHERE m.room_id = $1 AND m.created_at < $2
		ORDER BY m.created_at DESC LIMIT $3
	`, roomID, before, limit)
	if err != nil {
		return nil, fmt.Errorf("failed to get messages: %w", err)
	}
	defer rows.Close()

	var messages []models.MessageWithSender
	for rows.Next() {
		var m models.MessageWithSender
		if err := rows.Scan(&m.ID, &m.RoomID, &m.SenderID, &m.Content, &m.CreatedAt, &m.SenderUsername); err != nil {
			return nil, err
		}
		messages = append(messages, m)
	}
	if messages == nil {
		messages = []models.MessageWithSender{}
	}
	return messages, nil
}

// --- Read Tracking ---

func UpdateLastRead(db *sql.DB, roomID, userID string, timestamp time.Time) error {
	_, err := db.Exec(
		`UPDATE room_members SET last_read_at = $1 WHERE room_id = $2 AND user_id = $3`,
		timestamp, roomID, userID,
	)
	return err
}

func GetUnreadCount(db *sql.DB, roomID, userID string) (int, error) {
	var count int
	err := db.QueryRow(`
		SELECT COUNT(*) FROM messages m
		JOIN room_members rm ON rm.room_id = m.room_id AND rm.user_id = $2
		WHERE m.room_id = $1 AND m.created_at > rm.last_read_at AND m.sender_id != $2
	`, roomID, userID).Scan(&count)
	return count, err
}
