import { apiGet, apiPost, apiDelete } from './client';
import type { Room, RoomResponse } from '../types';

export async function getRooms(): Promise<Room[]> {
  return apiGet<Room[]>('/rooms');
}

export async function createRoom(name: string): Promise<RoomResponse> {
  return apiPost<RoomResponse>('/rooms', { name });
}

export async function getRoom(id: string): Promise<RoomResponse> {
  return apiGet<RoomResponse>(`/rooms/${id}`);
}

export async function joinRoom(id: string): Promise<void> {
  return apiPost<void>(`/rooms/${id}/join`);
}

export async function leaveRoom(id: string): Promise<void> {
  return apiDelete<void>(`/rooms/${id}/leave`);
}

export async function getDMs(): Promise<Room[]> {
  return apiGet<Room[]>('/dm');
}

export async function startDM(userId: string): Promise<RoomResponse> {
  return apiPost<RoomResponse>('/dm', { user_id: userId });
}

export async function searchUsers(query: string): Promise<{ users: Array<{ id: string; username: string; email: string }> }> {
  return apiGet<{ users: Array<{ id: string; username: string; email: string }> }>(`/users/search?q=${encodeURIComponent(query)}`);
}
