import type { Room } from '../../types';
import Avatar from '../common/Avatar';
import Badge from '../common/Badge';

interface RoomItemProps {
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

export default function RoomItem({ room, isActive, onClick }: RoomItemProps) {
  return (
    <button
      onClick={onClick}
      className={`w-full flex items-center gap-3 px-3 py-3 rounded-lg transition-sidebar text-left ${
        isActive
          ? 'bg-blue-500/10 border border-blue-500/20'
          : 'hover:bg-slate-700/50 border border-transparent'
      }`}
    >
      <Avatar username={room.name} size="md" />
      <div className="flex-1 min-w-0">
        <div className="flex items-center justify-between">
          <span
            className={`text-sm font-semibold truncate ${
              isActive ? 'text-white' : 'text-slate-200'
            }`}
          >
            # {room.name}
          </span>
          {room.last_message_at && (
            <span className="text-xs text-slate-500 flex-shrink-0 ml-2">
              {formatTime(room.last_message_at)}
            </span>
          )}
        </div>
        <div className="flex items-center justify-between mt-0.5">
          <p className="text-xs text-slate-400 truncate">
            {truncate(room.last_message, 40) || 'No messages yet'}
          </p>
          <Badge count={room.unread_count} className="ml-2 flex-shrink-0" />
        </div>
      </div>
    </button>
  );
}
