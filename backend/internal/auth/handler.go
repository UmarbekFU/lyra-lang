package auth

import (
	"database/sql"
	"encoding/json"
	"log/slog"
	"net/http"
	"strings"

	"github.com/umar/day22-chat/internal/database"
	"golang.org/x/crypto/bcrypt"
)

type registerRequest struct {
	Username string `json:"username"`
	Email    string `json:"email"`
	Password string `json:"password"`
}

type loginRequest struct {
	Username string `json:"username"`
	Password string `json:"password"`
}

type authResponse struct {
	Token string      `json:"token"`
	User  interface{} `json:"user"`
}

func writeJSON(w http.ResponseWriter, status int, data interface{}) {
	w.Header().Set("Content-Type", "application/json")
	w.WriteHeader(status)
	json.NewEncoder(w).Encode(data)
}

func writeError(w http.ResponseWriter, status int, msg string) {
	writeJSON(w, status, map[string]string{"error": msg})
}

func RegisterHandler(db *sql.DB, jwtSecret string) http.HandlerFunc {
	return func(w http.ResponseWriter, r *http.Request) {
		var req registerRequest
		if err := json.NewDecoder(r.Body).Decode(&req); err != nil {
			writeError(w, http.StatusBadRequest, "invalid request body")
			return
		}

		req.Username = strings.TrimSpace(req.Username)
		req.Email = strings.TrimSpace(req.Email)

		if req.Username == "" || req.Email == "" || req.Password == "" {
			writeError(w, http.StatusBadRequest, "username, email, and password are required")
			return
		}
		if len(req.Password) < 6 {
			writeError(w, http.StatusBadRequest, "password must be at least 6 characters")
			return
		}

		hash, err := bcrypt.GenerateFromPassword([]byte(req.Password), 12)
		if err != nil {
			slog.Error("failed to hash password", "error", err)
			writeError(w, http.StatusInternalServerError, "internal error")
			return
		}

		user, err := database.CreateUser(db, req.Username, req.Email, string(hash))
		if err != nil {
			if strings.Contains(err.Error(), "duplicate") {
				writeError(w, http.StatusConflict, "username or email already exists")
				return
			}
			slog.Error("failed to create user", "error", err)
			writeError(w, http.StatusInternalServerError, "internal error")
			return
		}

		token, err := GenerateToken(user.ID, user.Username, jwtSecret)
		if err != nil {
			slog.Error("failed to generate token", "error", err)
			writeError(w, http.StatusInternalServerError, "internal error")
			return
		}

		writeJSON(w, http.StatusCreated, authResponse{Token: token, User: user})
	}
}

func LoginHandler(db *sql.DB, jwtSecret string) http.HandlerFunc {
	return func(w http.ResponseWriter, r *http.Request) {
		var req loginRequest
		if err := json.NewDecoder(r.Body).Decode(&req); err != nil {
			writeError(w, http.StatusBadRequest, "invalid request body")
			return
		}

		if req.Username == "" || req.Password == "" {
			writeError(w, http.StatusBadRequest, "username and password are required")
			return
		}

		user, err := database.GetUserByUsername(db, req.Username)
		if err != nil {
			slog.Error("failed to get user", "error", err)
			writeError(w, http.StatusInternalServerError, "internal error")
			return
		}
		if user == nil {
			writeError(w, http.StatusUnauthorized, "invalid username or password")
			return
		}

		if err := bcrypt.CompareHashAndPassword([]byte(user.Password), []byte(req.Password)); err != nil {
			writeError(w, http.StatusUnauthorized, "invalid username or password")
			return
		}

		token, err := GenerateToken(user.ID, user.Username, jwtSecret)
		if err != nil {
			slog.Error("failed to generate token", "error", err)
			writeError(w, http.StatusInternalServerError, "internal error")
			return
		}

		user.Password = ""
		writeJSON(w, http.StatusOK, authResponse{Token: token, User: user})
	}
}

func MeHandler(db *sql.DB) http.HandlerFunc {
	return func(w http.ResponseWriter, r *http.Request) {
		userID := r.Context().Value(UserIDKey).(string)
		user, err := database.GetUserByID(db, userID)
		if err != nil || user == nil {
			writeError(w, http.StatusNotFound, "user not found")
			return
		}
		writeJSON(w, http.StatusOK, user)
	}
}
