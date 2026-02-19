package handlers

import (
	"database/sql"
	"encoding/json"
	"log/slog"
	"net/http"

	"github.com/umar/day22-chat/internal/auth"
	"github.com/umar/day22-chat/internal/database"
)

func SearchUsersHandler(db *sql.DB) http.HandlerFunc {
	return func(w http.ResponseWriter, r *http.Request) {
		query := r.URL.Query().Get("q")
		if query == "" {
			writeJSON(w, http.StatusOK, []interface{}{})
			return
		}

		users, err := database.SearchUsers(db, query, 20)
		if err != nil {
			slog.Error("failed to search users", "error", err)
			writeError(w, http.StatusInternalServerError, "internal error")
			return
		}

		writeJSON(w, http.StatusOK, users)
	}
}

func StartDM(db *sql.DB) http.HandlerFunc {
	return func(w http.ResponseWriter, r *http.Request) {
		userID := r.Context().Value(auth.UserIDKey).(string)

		var req struct {
			UserID string `json:"user_id"`
		}
		if err := json.NewDecoder(r.Body).Decode(&req); err != nil {
			writeError(w, http.StatusBadRequest, "invalid request body")
			return
		}
		if req.UserID == "" {
			writeError(w, http.StatusBadRequest, "user_id is required")
			return
		}

		room, err := database.GetOrCreateDMRoom(db, userID, req.UserID)
		if err != nil {
			slog.Error("failed to create DM", "error", err)
			writeError(w, http.StatusInternalServerError, "internal error")
			return
		}

		writeJSON(w, http.StatusOK, room)
	}
}

func ListDMs(db *sql.DB) http.HandlerFunc {
	return func(w http.ResponseWriter, r *http.Request) {
		userID := r.Context().Value(auth.UserIDKey).(string)
		rooms, err := database.GetDMRooms(db, userID)
		if err != nil {
			slog.Error("failed to list DMs", "error", err)
			writeError(w, http.StatusInternalServerError, "internal error")
			return
		}
		writeJSON(w, http.StatusOK, rooms)
	}
}
