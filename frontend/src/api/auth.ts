import { apiPost, apiGet } from './client';
import type { AuthResponse, User } from '../types';

export async function login(username: string, password: string): Promise<AuthResponse> {
  return apiPost<AuthResponse>('/auth/login', { username, password });
}

export async function register(username: string, email: string, password: string): Promise<AuthResponse> {
  return apiPost<AuthResponse>('/auth/register', { username, email, password });
}

export async function getMe(): Promise<{ user: User }> {
  return apiGet<{ user: User }>('/auth/me');
}
