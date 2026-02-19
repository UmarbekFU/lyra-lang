import type { Room } from '../../types';
import { useAuth } from '../../context/AuthContext';
import { useChat } from '../../context/ChatContext';
import Avatar from '../common/Avatar';
import Badge from '../common/Badge';
import OnlineIndicator from '../common/OnlineIndicator';

interface DMItemProps {
  room: Room;
  isActive: boolean;
  onClick: () => void;
}

function formatTime(dateStr: string | null): string {
  if (!dateStr) return '';
  const date = new Date(dateStr);
  const now = new Date();
  const diff = now.getTime() - date.getTime();
  const dayMs = 86400000;

  if (diff < dayMs && date.getDate() === now.getDate()) {
    return date.toLocaleTimeString([], { hour: 'numeric', minute: '2-digit' });
  }
  if (diff < dayMs * 2) {
    return 'Yesterday';
  }
  if (diff < dayMs * 7) {
    return date.toLocaleDateString([], { weekday: 'short' });
  }
  return date.toLocaleDateString([], { month: 'short', day: 'numeric' });
}

function truncate(str: string, maxLen: number): string {
  if (!str) return '';
  return str.length > maxLen ? str.slice(0, maxLen) + '...' : str;
}

function getOtherUsername(roomName: string, currentUsername: string): string {
  // DM room names are usually "user1:user2" or similar
  const parts = roomName.split(':');
  if (parts.length === 2) {
    return parts[0] === currentUsername ? parts[1] : parts[0];
  }
  // Fallback: if name contains the current user, remove it
  const nameWithoutCurrent = roomName.replace(currentUsername, '').replace(/[^a-zA-Z0-9]/g, '');
  return nameWithoutCurrent || roomName;
}

export default function DMItem({ room, isActive, onClick }: DMItemProps) {
  const { user } = useAuth();
  const { onlineUsers } = useChat();

  const otherUsername = getOtherUsername(room.name, user?.username || '');

  // Try to find the other user's ID from members
  const otherUser = room.members?.find((m) => m.id !== user?.id);
  const isOnline = otherUser ? onlineUsers.has(otherUser.id) : false;

  return (
    <button
      onClick={onClick}
      className={`w-full flex items-center gap-3 px-3 py-3 rounded-lg transition-sidebar text-left ${
        isActive
          ? 'bg-blue-500/10 border border-blue-500/20'
          : 'hover:bg-slate-700/50 border border-transparent'
      }`}
    >
      <div className="relative flex-shrink-0">
        <Avatar username={otherUsername} size="md" />
        <div className="absolute -bottom-0.5 -right-0.5">
          <OnlineIndicator isOnline={isOnline} />
        </div>
      </div>
      <div className="flex-1 min-w-0">
        <div className="flex items-center justify-between">
          <span
            className={`text-sm font-semibold truncate ${
              isActive ? 'text-white' : 'text-slate-200'
            }`}
          >
            {otherUsername}
          </span>
          {room.last_message_at && (
            <span className="text-xs text-slate-500 flex-shrink-0 ml-2">
              {formatTime(room.last_message_at)}
            </span>
          )}
        </div>
        <div className="flex items-center justify-between mt-0.5">
          <p className="text-xs text-slate-400 truncate">
            {truncate(room.last_message, 40) || 'Start a conversation'}
          </p>
          <Badge count={room.unread_count} className="ml-2 flex-shrink-0" />
        </div>
      </div>
    </button>
  );
}
