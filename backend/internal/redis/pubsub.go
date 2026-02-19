package redisc

import (
	"context"
	"log/slog"

	"github.com/redis/go-redis/v9"
)

func PublishToRoom(client *redis.Client, roomID string, data []byte) error {
	return client.Publish(context.Background(), "chat:room:"+roomID, data).Err()
}

func SubscribeRooms(client *redis.Client, handler func(roomID string, data []byte)) {
	ctx := context.Background()
	pubsub := client.PSubscribe(ctx, "chat:room:*")
	defer pubsub.Close()

	ch := pubsub.Channel()
	for msg := range ch {
		roomID := msg.Channel[len("chat:room:"):]
		handler(roomID, []byte(msg.Payload))
		slog.Debug("pubsub message", "room_id", roomID)
	}
}
