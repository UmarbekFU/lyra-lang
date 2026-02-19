package redisc

import (
	"context"
	"time"

	"github.com/redis/go-redis/v9"
)

const presenceTTL = 120 * time.Second

func SetOnline(client *redis.Client, userID string) error {
	ctx := context.Background()
	pipe := client.Pipeline()
	pipe.SAdd(ctx, "online_users", userID)
	pipe.Set(ctx, "presence:"+userID, "online", presenceTTL)
	_, err := pipe.Exec(ctx)
	return err
}

func SetOffline(client *redis.Client, userID string) error {
	ctx := context.Background()
	pipe := client.Pipeline()
	pipe.SRem(ctx, "online_users", userID)
	pipe.Del(ctx, "presence:"+userID)
	_, err := pipe.Exec(ctx)
	return err
}

func GetOnlineUsers(client *redis.Client) ([]string, error) {
	return client.SMembers(context.Background(), "online_users").Result()
}

func RefreshPresence(client *redis.Client, userID string) error {
	return client.Expire(context.Background(), "presence:"+userID, presenceTTL).Err()
}
