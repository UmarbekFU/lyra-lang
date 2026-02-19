export interface User {
  id: string;
  username: string;
  email: string;
  avatar_url: string;
  created_at: string;
}

export interface Room {
  id: string;
  name: string;
  type: 'group' | 'dm';
  created_by: string;
  created_at: string;
  unread_count: number;
  last_message: string;
  last_message_at: string | null;
  members?: User[];
}

export interface Message {
  id: string;
  room_id: string;
  sender_id: string;
  sender_username: string;
  content: string;
  created_at: string;
}

export interface WSMessage {
  type: string;
  payload: any;
}

export interface TypingUser {
  user_id: string;
  username: string;
}

export interface AuthResponse {
  token: string;
  user: User;
}

export interface RoomResponse {
  room: Room;
  members?: User[];
}

export interface ApiError {
  error: string;
}
