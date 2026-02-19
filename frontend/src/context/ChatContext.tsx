import {
  createContext,
  useContext,
  useState,
  useCallback,
  useEffect,
  useRef,
  type ReactNode,
} from 'react';
import type { Room, Message, TypingUser } from '../types';
import { useAuth } from './AuthContext';
import { useWebSocket } from '../hooks/useWebSocket';
import * as roomsApi from '../api/rooms';
import * as messagesApi from '../api/messages';

interface ChatContextType {
  rooms: Room[];
  activeRoom: Room | null;
  setActiveRoom: (room: Room | null) => void;
  messages: Record<string, Message[]>;
  typingUsers: Record<string, TypingUser[]>;
  onlineUsers: Set<string>;
  sendMessage: (roomId: string, content: string) => void;
  markAsRead: (roomId: string) => void;
  startTyping: (roomId: string) => void;
  stopTyping: (roomId: string) => void;
  createRoom: (name: string) => Promise<void>;
  joinRoom: (id: string) => void;
  loadMoreMessages: (roomId: string) => Promise<boolean>;
  isConnected: boolean;
}

const ChatContext = createContext<ChatContextType | null>(null);

export function ChatProvider({ children }: { children: ReactNode }) {
  const { user, token } = useAuth();
  const ws = useWebSocket();

  const [rooms, setRooms] = useState<Room[]>([]);
  const [activeRoom, setActiveRoomState] = useState<Room | null>(null);
  const [messages, setMessages] = useState<Record<string, Message[]>>({});
  const [typingUsers, setTypingUsers] = useState<Record<string, TypingUser[]>>({});
  const [onlineUsers, setOnlineUsers] = useState<Set<string>>(new Set());
  const [hasMoreMessages, setHasMoreMessages] = useState<Record<string, boolean>>({});

  const activeRoomRef = useRef<Room | null>(null);

  const setActiveRoom = useCallback((room: Room | null) => {
    setActiveRoomState(room);
    activeRoomRef.current = room;

    if (room) {
      // Join the room via WS
      ws.send('room.join', { room_id: room.id });

      // Mark as read
      ws.send('message.read', { room_id: room.id });

      // Clear unread count locally
      setRooms((prev) =>
        prev.map((r) => (r.id === room.id ? { ...r, unread_count: 0 } : r))
      );

      // Load messages if not already loaded
      if (!messages[room.id]) {
        messagesApi.getMessages(room.id).then((msgs) => {
          setMessages((prev) => ({ ...prev, [room.id]: msgs }));
          setHasMoreMessages((prev) => ({ ...prev, [room.id]: msgs.length === 50 }));
        });
      }
    }
  }, [ws, messages]);

  const sendMessage = useCallback(
    (roomId: string, content: string) => {
      ws.send('message.send', { room_id: roomId, content });
    },
    [ws]
  );

  const markAsRead = useCallback(
    (roomId: string) => {
      ws.send('message.read', { room_id: roomId });
      setRooms((prev) =>
        prev.map((r) => (r.id === roomId ? { ...r, unread_count: 0 } : r))
      );
    },
    [ws]
  );

  const startTyping = useCallback(
    (roomId: string) => {
      ws.send('typing.start', { room_id: roomId });
    },
    [ws]
  );

  const stopTyping = useCallback(
    (roomId: string) => {
      ws.send('typing.stop', { room_id: roomId });
    },
    [ws]
  );

  const createRoom = useCallback(async (name: string) => {
    const response = await roomsApi.createRoom(name);
    setRooms((prev) => [response.room, ...prev]);
  }, []);

  const joinRoom = useCallback(
    (id: string) => {
      ws.send('room.join', { room_id: id });
      roomsApi.joinRoom(id);
    },
    [ws]
  );

  const loadMoreMessages = useCallback(
    async (roomId: string): Promise<boolean> => {
      const roomMessages = messages[roomId];
      if (!roomMessages || roomMessages.length === 0) return false;
      if (hasMoreMessages[roomId] === false) return false;

      const oldest = roomMessages[0];
      const olderMsgs = await messagesApi.getMessages(roomId, oldest.created_at);

      if (olderMsgs.length === 0) {
        setHasMoreMessages((prev) => ({ ...prev, [roomId]: false }));
        return false;
      }

      setMessages((prev) => ({
        ...prev,
        [roomId]: [...olderMsgs, ...(prev[roomId] || [])],
      }));

      setHasMoreMessages((prev) => ({
        ...prev,
        [roomId]: olderMsgs.length === 50,
      }));

      return olderMsgs.length > 0;
    },
    [messages, hasMoreMessages]
  );

  // Connect WebSocket when token is available
  useEffect(() => {
    if (token) {
      ws.connect(token);
    }
    return () => {
      ws.disconnect();
    };
    // Only reconnect when token changes
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [token]);

  // Fetch rooms on mount
  useEffect(() => {
    if (!token) return;

    const fetchRooms = async () => {
      try {
        const [groupRooms, dmRooms] = await Promise.all([
          roomsApi.getRooms(),
          roomsApi.getDMs(),
        ]);
        const allRooms = [...(groupRooms || []), ...(dmRooms || [])];
        setRooms(allRooms);
      } catch (err) {
        console.error('Failed to fetch rooms:', err);
      }
    };

    fetchRooms();
  }, [token]);

  // Subscribe to WebSocket events
  useEffect(() => {
    const unsubs: Array<() => void> = [];

    unsubs.push(
      ws.on('message.new', (payload: Message) => {
        // Add message to the room
        setMessages((prev) => {
          const roomMsgs = prev[payload.room_id] || [];
          // Prevent duplicates
          if (roomMsgs.some((m) => m.id === payload.id)) return prev;
          return {
            ...prev,
            [payload.room_id]: [...roomMsgs, payload],
          };
        });

        // Update room's last_message
        setRooms((prev) =>
          prev.map((r) => {
            if (r.id === payload.room_id) {
              const isActive = activeRoomRef.current?.id === payload.room_id;
              return {
                ...r,
                last_message: payload.content,
                last_message_at: payload.created_at,
                unread_count: isActive ? r.unread_count : r.unread_count + 1,
              };
            }
            return r;
          })
        );

        // If active room, mark as read
        if (activeRoomRef.current?.id === payload.room_id && payload.sender_id !== user?.id) {
          ws.send('message.read', { room_id: payload.room_id });
        }
      })
    );

    unsubs.push(
      ws.on('typing.update', (payload: { room_id: string; users: TypingUser[] }) => {
        setTypingUsers((prev) => ({
          ...prev,
          [payload.room_id]: (payload.users || []).filter(
            (u: TypingUser) => u.user_id !== user?.id
          ),
        }));
      })
    );

    unsubs.push(
      ws.on('presence.update', (payload: { user_id: string; status: string }) => {
        setOnlineUsers((prev) => {
          const next = new Set(prev);
          if (payload.status === 'online') {
            next.add(payload.user_id);
          } else {
            next.delete(payload.user_id);
          }
          return next;
        });
      })
    );

    unsubs.push(
      ws.on('unread.update', (payload: { room_id: string; unread_count: number }) => {
        setRooms((prev) =>
          prev.map((r) =>
            r.id === payload.room_id ? { ...r, unread_count: payload.unread_count } : r
          )
        );
      })
    );

    unsubs.push(
      ws.on('read_receipt.update', (_payload: { room_id: string; user_id: string; read_at: string }) => {
        // Read receipts handled in UI component
      })
    );

    unsubs.push(
      ws.on('room.member_joined', (payload: { room_id: string; user_id: string; username: string }) => {
        // Could add a system message
        setMessages((prev) => {
          const roomMsgs = prev[payload.room_id] || [];
          const systemMsg: Message = {
            id: `system-join-${payload.user_id}-${Date.now()}`,
            room_id: payload.room_id,
            sender_id: 'system',
            sender_username: 'system',
            content: `${payload.username} joined the room`,
            created_at: new Date().toISOString(),
          };
          return { ...prev, [payload.room_id]: [...roomMsgs, systemMsg] };
        });
      })
    );

    unsubs.push(
      ws.on('room.member_left', (payload: { room_id: string; user_id: string; username: string }) => {
        setMessages((prev) => {
          const roomMsgs = prev[payload.room_id] || [];
          const systemMsg: Message = {
            id: `system-leave-${payload.user_id}-${Date.now()}`,
            room_id: payload.room_id,
            sender_id: 'system',
            sender_username: 'system',
            content: `${payload.username} left the room`,
            created_at: new Date().toISOString(),
          };
          return { ...prev, [payload.room_id]: [...roomMsgs, systemMsg] };
        });
      })
    );

    unsubs.push(
      ws.on('error', (payload: { message: string }) => {
        console.error('WebSocket error:', payload.message);
      })
    );

    unsubs.push(ws.on('pong', () => {}));

    return () => {
      unsubs.forEach((unsub) => unsub());
    };
  }, [ws, user?.id]);

  return (
    <ChatContext.Provider
      value={{
        rooms,
        activeRoom,
        setActiveRoom,
        messages,
        typingUsers,
        onlineUsers,
        sendMessage,
        markAsRead,
        startTyping,
        stopTyping,
        createRoom,
        joinRoom,
        loadMoreMessages,
        isConnected: ws.isConnected,
      }}
    >
      {children}
    </ChatContext.Provider>
  );
}

export function useChat(): ChatContextType {
  const context = useContext(ChatContext);
  if (!context) {
    throw new Error('useChat must be used within a ChatProvider');
  }
  return context;
}
