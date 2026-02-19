package handlers

import (
	"database/sql"
	"log/slog"
	"net/http"
	"strconv"
	"time"

	"github.com/gorilla/mux"
	"github.com/umar/day22-chat/internal/auth"
	"github.com/umar/day22-chat/internal/database"
)

func GetMessages(db *sql.DB) http.HandlerFunc {
	return func(w http.ResponseWriter, r *http.Request) {
		roomID := mux.Vars(r)["id"]
		userID := r.Context().Value(auth.UserIDKey).(string)

		isMember, err := database.IsRoomMember(db, roomID, userID)
		if err != nil || !isMember {
			writeError(w, http.StatusForbidden, "not a member of this room")
			return
		}

		before := time.Now()
		if beforeStr := r.URL.Query().Get("before"); beforeStr != "" {
			if t, err := time.Parse(time.RFC3339, beforeStr); err == nil {
				before = t
			}
		}

		limit := 50
		if limitStr := r.URL.Query().Get("limit"); limitStr != "" {
			if l, err := strconv.Atoi(limitStr); err == nil && l > 0 && l <= 100 {
				limit = l
			}
		}

		messages, err := database.GetMessages(db, roomID, before, limit)
		if err != nil {
			slog.Error("failed to get messages", "error", err)
			writeError(w, http.StatusInternalServerError, "internal error")
			return
		}

		writeJSON(w, http.StatusOK, messages)
	}
}
