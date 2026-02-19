package handlers

import (
	"database/sql"
	"encoding/json"
	"log/slog"
	"net/http"

	"github.com/gorilla/mux"
	"github.com/umar/day22-chat/internal/auth"
	"github.com/umar/day22-chat/internal/database"
)

func ListRooms(db *sql.DB) http.HandlerFunc {
	return func(w http.ResponseWriter, r *http.Request) {
		userID := r.Context().Value(auth.UserIDKey).(string)
		rooms, err := database.GetRoomsForUser(db, userID)
		if err != nil {
			slog.Error("failed to list rooms", "error", err)
			writeError(w, http.StatusInternalServerError, "internal error")
			return
		}
		writeJSON(w, http.StatusOK, rooms)
	}
}

func CreateRoom(db *sql.DB) http.HandlerFunc {
	return func(w http.ResponseWriter, r *http.Request) {
		userID := r.Context().Value(auth.UserIDKey).(string)

		var req struct {
			Name string `json:"name"`
		}
		if err := json.NewDecoder(r.Body).Decode(&req); err != nil {
			writeError(w, http.StatusBadRequest, "invalid request body")
			return
		}
		if req.Name == "" {
			writeError(w, http.StatusBadRequest, "name is required")
			return
		}

		room, err := database.CreateRoom(db, req.Name, "group", userID)
		if err != nil {
			slog.Error("failed to create room", "error", err)
			writeError(w, http.StatusInternalServerError, "internal error")
			return
		}

		if err := database.AddRoomMember(db, room.ID, userID); err != nil {
			slog.Error("failed to add creator to room", "error", err)
		}

		writeJSON(w, http.StatusCreated, room)
	}
}

func GetRoom(db *sql.DB) http.HandlerFunc {
	return func(w http.ResponseWriter, r *http.Request) {
		roomID := mux.Vars(r)["id"]
		userID := r.Context().Value(auth.UserIDKey).(string)

		isMember, err := database.IsRoomMember(db, roomID, userID)
		if err != nil || !isMember {
			writeError(w, http.StatusForbidden, "not a member of this room")
			return
		}

		room, err := database.GetRoomByID(db, roomID)
		if err != nil || room == nil {
			writeError(w, http.StatusNotFound, "room not found")
			return
		}

		members, err := database.GetRoomMembers(db, roomID)
		if err != nil {
			members = nil
		}

		writeJSON(w, http.StatusOK, map[string]interface{}{
			"room":    room,
			"members": members,
		})
	}
}

func JoinRoom(db *sql.DB) http.HandlerFunc {
	return func(w http.ResponseWriter, r *http.Request) {
		roomID := mux.Vars(r)["id"]
		userID := r.Context().Value(auth.UserIDKey).(string)

		room, err := database.GetRoomByID(db, roomID)
		if err != nil || room == nil {
			writeError(w, http.StatusNotFound, "room not found")
			return
		}

		if err := database.AddRoomMember(db, roomID, userID); err != nil {
			slog.Error("failed to join room", "error", err)
			writeError(w, http.StatusInternalServerError, "internal error")
			return
		}

		writeJSON(w, http.StatusOK, map[string]string{"status": "joined"})
	}
}

func LeaveRoom(db *sql.DB) http.HandlerFunc {
	return func(w http.ResponseWriter, r *http.Request) {
		roomID := mux.Vars(r)["id"]
		userID := r.Context().Value(auth.UserIDKey).(string)

		if err := database.RemoveRoomMember(db, roomID, userID); err != nil {
			slog.Error("failed to leave room", "error", err)
			writeError(w, http.StatusInternalServerError, "internal error")
			return
		}

		writeJSON(w, http.StatusOK, map[string]string{"status": "left"})
	}
}
