import type { TypingUser } from '../../types';

interface TypingIndicatorProps {
  typingUsers: TypingUser[];
}

export default function TypingIndicator({ typingUsers }: TypingIndicatorProps) {
  if (!typingUsers || typingUsers.length === 0) return null;

  let text: string;
  if (typingUsers.length === 1) {
    text = `${typingUsers[0].username} is typing`;
  } else if (typingUsers.length === 2) {
    text = `${typingUsers[0].username} and ${typingUsers[1].username} are typing`;
  } else {
    text = `${typingUsers[0].username} and ${typingUsers.length - 1} others are typing`;
  }

  return (
    <div className="flex items-center gap-2 px-4 py-2">
      <div className="flex items-center gap-1">
        <span className="typing-dot w-1.5 h-1.5 bg-slate-400 rounded-full inline-block" />
        <span className="typing-dot w-1.5 h-1.5 bg-slate-400 rounded-full inline-block" />
        <span className="typing-dot w-1.5 h-1.5 bg-slate-400 rounded-full inline-block" />
      </div>
      <span className="text-xs text-slate-400">{text}</span>
    </div>
  );
}
