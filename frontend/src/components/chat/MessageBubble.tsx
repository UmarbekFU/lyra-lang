import type { Message } from '../../types';
import Avatar from '../common/Avatar';

interface MessageBubbleProps {
  message: Message;
  isOwn: boolean;
  showSender: boolean;
  isGroupChat: boolean;
}

function formatTime(dateStr: string): string {
  const date = new Date(dateStr);
  return date.toLocaleTimeString([], { hour: 'numeric', minute: '2-digit' });
}

export default function MessageBubble({
  message,
  isOwn,
  showSender,
  isGroupChat,
}: MessageBubbleProps) {
  // System messages
  if (message.sender_id === 'system') {
    return (
      <div className="flex justify-center py-1">
        <span className="text-xs text-slate-500 bg-slate-800/50 px-3 py-1 rounded-full">
          {message.content}
        </span>
      </div>
    );
  }

  return (
    <div
      className={`flex items-end gap-2 message-enter ${
        isOwn ? 'flex-row-reverse' : 'flex-row'
      } ${showSender ? 'mt-3' : 'mt-0.5'}`}
    >
      {/* Avatar */}
      <div className="w-8 flex-shrink-0">
        {showSender && !isOwn && (
          <Avatar username={message.sender_username} size="sm" />
        )}
      </div>

      {/* Bubble */}
      <div className={`max-w-[70%] ${isOwn ? 'items-end' : 'items-start'}`}>
        {/* Sender name */}
        {showSender && !isOwn && isGroupChat && (
          <p className="text-xs font-medium text-slate-400 mb-1 ml-1">
            {message.sender_username}
          </p>
        )}

        <div
          className={`px-4 py-2.5 rounded-2xl break-words ${
            isOwn
              ? 'bg-blue-500 text-white rounded-br-md'
              : 'bg-[#334155] text-slate-100 rounded-bl-md'
          }`}
        >
          <p className="text-sm whitespace-pre-wrap leading-relaxed">{message.content}</p>
        </div>

        {/* Timestamp */}
        <p
          className={`text-[10px] text-slate-500 mt-1 ${
            isOwn ? 'text-right mr-1' : 'ml-1'
          }`}
        >
          {formatTime(message.created_at)}
        </p>
      </div>
    </div>
  );
}
