import { useState, useRef, useCallback, useEffect, type KeyboardEvent } from 'react';
import { useChat } from '../../context/ChatContext';

export default function MessageInput() {
  const [content, setContent] = useState('');
  const { activeRoom, sendMessage, startTyping, stopTyping } = useChat();
  const textareaRef = useRef<HTMLTextAreaElement>(null);
  const typingTimerRef = useRef<ReturnType<typeof setTimeout> | null>(null);
  const isTypingRef = useRef(false);

  const adjustHeight = useCallback(() => {
    const textarea = textareaRef.current;
    if (!textarea) return;
    textarea.style.height = 'auto';
    const lineHeight = 24;
    const maxHeight = lineHeight * 5;
    textarea.style.height = `${Math.min(textarea.scrollHeight, maxHeight)}px`;
  }, []);

  useEffect(() => {
    adjustHeight();
  }, [content, adjustHeight]);

  // Focus textarea when active room changes
  useEffect(() => {
    if (activeRoom && textareaRef.current) {
      textareaRef.current.focus();
    }
  }, [activeRoom]);

  const handleTyping = useCallback(() => {
    if (!activeRoom) return;

    if (!isTypingRef.current) {
      isTypingRef.current = true;
      startTyping(activeRoom.id);
    }

    // Reset the stop timer
    if (typingTimerRef.current) {
      clearTimeout(typingTimerRef.current);
    }

    typingTimerRef.current = setTimeout(() => {
      if (activeRoom && isTypingRef.current) {
        isTypingRef.current = false;
        stopTyping(activeRoom.id);
      }
    }, 2000);
  }, [activeRoom, startTyping, stopTyping]);

  const handleSend = useCallback(() => {
    const trimmed = content.trim();
    if (!trimmed || !activeRoom) return;

    sendMessage(activeRoom.id, trimmed);
    setContent('');

    // Stop typing indicator
    if (typingTimerRef.current) {
      clearTimeout(typingTimerRef.current);
    }
    if (isTypingRef.current) {
      isTypingRef.current = false;
      stopTyping(activeRoom.id);
    }

    // Reset textarea height
    if (textareaRef.current) {
      textareaRef.current.style.height = 'auto';
    }
  }, [content, activeRoom, sendMessage, stopTyping]);

  const handleKeyDown = (e: KeyboardEvent<HTMLTextAreaElement>) => {
    if (e.key === 'Enter' && !e.shiftKey) {
      e.preventDefault();
      handleSend();
    }
  };

  const handleChange = (value: string) => {
    setContent(value);
    if (value.trim()) {
      handleTyping();
    }
  };

  // Cleanup on unmount
  useEffect(() => {
    return () => {
      if (typingTimerRef.current) {
        clearTimeout(typingTimerRef.current);
      }
    };
  }, []);

  const disabled = !activeRoom;

  return (
    <div className="px-4 py-3 border-t border-slate-700/50 bg-[#1e293b]">
      <div className="flex items-end gap-3">
        <div className="flex-1 relative">
          <textarea
            ref={textareaRef}
            value={content}
            onChange={(e) => handleChange(e.target.value)}
            onKeyDown={handleKeyDown}
            placeholder={disabled ? 'Select a room to start chatting' : 'Type a message...'}
            disabled={disabled}
            rows={1}
            className="w-full px-4 py-3 rounded-xl bg-[#0f172a] border border-slate-600/50 text-white placeholder-slate-500 focus:outline-none focus:ring-2 focus:ring-blue-500/50 focus:border-transparent transition-all resize-none text-sm leading-6 disabled:opacity-50 disabled:cursor-not-allowed"
          />
        </div>
        <button
          onClick={handleSend}
          disabled={disabled || !content.trim()}
          className="flex-shrink-0 p-3 rounded-xl bg-blue-500 hover:bg-blue-600 text-white transition-colors disabled:opacity-30 disabled:cursor-not-allowed disabled:hover:bg-blue-500"
        >
          <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path
              strokeLinecap="round"
              strokeLinejoin="round"
              strokeWidth={2}
              d="M12 19l9 2-9-18-9 18 9-2zm0 0v-8"
            />
          </svg>
        </button>
      </div>
    </div>
  );
}
