import { apiGet } from './client';
import type { Message } from '../types';

export async function getMessages(
  roomId: string,
  before?: string,
  limit: number = 50
): Promise<Message[]> {
  let path = `/rooms/${roomId}/messages?limit=${limit}`;
  if (before) {
    path += `&before=${encodeURIComponent(before)}`;
  }
  return apiGet<Message[]>(path);
}
