package main

import (
	"context"
	"log/slog"
	"net/http"
	"os"
	"os/signal"
	"syscall"
	"time"

	"github.com/gorilla/mux"
	"github.com/umar/day22-chat/internal/auth"
	"github.com/umar/day22-chat/internal/chat"
	"github.com/umar/day22-chat/internal/database"
	"github.com/umar/day22-chat/internal/handlers"
	"github.com/umar/day22-chat/internal/middleware"
	redisc "github.com/umar/day22-chat/internal/redis"
)

func getEnv(key, fallback string) string {
	if v, ok := os.LookupEnv(key); ok {
		return v
	}
	return fallback
}

func main() {
	logger := slog.New(slog.NewJSONHandler(os.Stdout, &slog.HandlerOptions{Level: slog.LevelInfo}))
	slog.SetDefault(logger)

	slog.Info("starting chat server")

	port := getEnv("PORT", "8080")
	databaseURL := getEnv("DATABASE_URL", "postgres://postgres:postgres@localhost:5432/chatapp?sslmode=disable")
	redisURL := getEnv("REDIS_URL", "redis://localhost:6379")
	jwtSecret := getEnv("JWT_SECRET", "dev-secret-change-me")
	corsOrigin := getEnv("CORS_ORIGIN", "http://localhost:5173")

	// Initialize database
	db, err := database.InitDB(databaseURL)
	if err != nil {
		slog.Error("failed to init database", "error", err)
		os.Exit(1)
	}
	defer db.Close()
	slog.Info("connected to PostgreSQL")

	if err := database.RunMigrations(db); err != nil {
		slog.Error("failed to run migrations", "error", err)
		os.Exit(1)
	}
	slog.Info("database migrations complete")

	// Initialize Redis
	redisClient, err := redisc.InitRedis(redisURL)
	if err != nil {
		slog.Error("failed to init Redis", "error", err)
		os.Exit(1)
	}
	defer redisClient.Close()
	slog.Info("connected to Redis")

	// Create WebSocket hub
	hub := chat.NewHub(db, redisClient)
	go hub.Run()

	// Set up router
	router := mux.NewRouter()
	router.Use(middleware.Logging)
	router.Use(middleware.CORS(corsOrigin))

	// Public routes
	router.HandleFunc("/health", handlers.Health).Methods("GET", "OPTIONS")
	router.HandleFunc("/api/auth/register", auth.RegisterHandler(db, jwtSecret)).Methods("POST", "OPTIONS")
	router.HandleFunc("/api/auth/login", auth.LoginHandler(db, jwtSecret)).Methods("POST", "OPTIONS")

	// WebSocket
	router.HandleFunc("/ws", chat.ServeWS(hub, db, jwtSecret)).Methods("GET")

	// Protected routes
	protected := router.PathPrefix("/api").Subrouter()
	protected.Use(auth.JWTMiddleware(jwtSecret))

	protected.HandleFunc("/auth/me", auth.MeHandler(db)).Methods("GET")
	protected.HandleFunc("/rooms", handlers.ListRooms(db)).Methods("GET")
	protected.HandleFunc("/rooms", handlers.CreateRoom(db)).Methods("POST")
	protected.HandleFunc("/rooms/{id}", handlers.GetRoom(db)).Methods("GET")
	protected.HandleFunc("/rooms/{id}/join", handlers.JoinRoom(db)).Methods("POST")
	protected.HandleFunc("/rooms/{id}/leave", handlers.LeaveRoom(db)).Methods("DELETE")
	protected.HandleFunc("/rooms/{id}/messages", handlers.GetMessages(db)).Methods("GET")
	protected.HandleFunc("/dm", handlers.StartDM(db)).Methods("POST")
	protected.HandleFunc("/dm", handlers.ListDMs(db)).Methods("GET")
	protected.HandleFunc("/users/search", handlers.SearchUsersHandler(db)).Methods("GET")

	// HTTP server
	srv := &http.Server{
		Addr:         ":" + port,
		Handler:      router,
		ReadTimeout:  15 * time.Second,
		WriteTimeout: 15 * time.Second,
		IdleTimeout:  120 * time.Second,
	}

	go func() {
		slog.Info("server listening", "port", port)
		if err := srv.ListenAndServe(); err != nil && err != http.ErrServerClosed {
			slog.Error("server failed", "error", err)
			os.Exit(1)
		}
	}()

	// Graceful shutdown
	quit := make(chan os.Signal, 1)
	signal.Notify(quit, syscall.SIGINT, syscall.SIGTERM)
	sig := <-quit
	slog.Info("shutting down", "signal", sig.String())

	ctx, cancel := context.WithTimeout(context.Background(), 30*time.Second)
	defer cancel()

	hub.Shutdown()
	if err := srv.Shutdown(ctx); err != nil {
		slog.Error("forced shutdown", "error", err)
		os.Exit(1)
	}

	slog.Info("server stopped gracefully")
}
