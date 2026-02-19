import { useEffect, useRef, useState, useCallback } from 'react';
import { useAuth } from '../../context/AuthContext';
import { useChat } from '../../context/ChatContext';
import MessageBubble from './MessageBubble';
import TypingIndicator from './TypingIndicator';
import ReadReceipt from './ReadReceipt';
import Spinner from '../common/Spinner';
import type { Message } from '../../types';

function isSameDay(d1: string, d2: string): boolean {
  const a = new Date(d1);
  const b = new Date(d2);
  return (
    a.getFullYear() === b.getFullYear() &&
    a.getMonth() === b.getMonth() &&
    a.getDate() === b.getDate()
  );
}

function formatDateSeparator(dateStr: string): string {
  const date = new Date(dateStr);
  const now = new Date();
  const yesterday = new Date(now);
  yesterday.setDate(yesterday.getDate() - 1);

  if (isSameDay(dateStr, now.toISOString())) {
    return 'Today';
  }
  if (isSameDay(dateStr, yesterday.toISOString())) {
    return 'Yesterday';
  }
  return date.toLocaleDateString([], {
    weekday: 'long',
    month: 'long',
    day: 'numeric',
    year: date.getFullYear() !== now.getFullYear() ? 'numeric' : undefined,
  });
}

export default function MessageList() {
  const { user } = useAuth();
  const { activeRoom, messages, typingUsers, loadMoreMessages } = useChat();
  const messagesEndRef = useRef<HTMLDivElement>(null);
  const scrollContainerRef = useRef<HTMLDivElement>(null);
  const topSentinelRef = useRef<HTMLDivElement>(null);
  const [isLoadingMore, setIsLoadingMore] = useState(false);
  const shouldAutoScrollRef = useRef(true);
  const prevMessagesLengthRef = useRef(0);

  const roomMessages: Message[] = activeRoom ? messages[activeRoom.id] || [] : [];
  const roomTypingUsers = activeRoom ? typingUsers[activeRoom.id] || [] : [];

  // Auto-scroll to bottom on new messages (only if already at bottom)
  useEffect(() => {
    if (!activeRoom) return;

    const currentLength = roomMessages.length;
    const prevLength = prevMessagesLengthRef.current;

    if (currentLength > prevLength && shouldAutoScrollRef.current) {
      messagesEndRef.current?.scrollIntoView({ behavior: currentLength - prevLength > 1 ? 'auto' : 'smooth' });
    }

    prevMessagesLengthRef.current = currentLength;
  }, [roomMessages, activeRoom]);

  // Scroll to bottom on room change
  useEffect(() => {
    if (activeRoom) {
      shouldAutoScrollRef.current = true;
      prevMessagesLengthRef.current = 0;
      setTimeout(() => {
        messagesEndRef.current?.scrollIntoView({ behavior: 'auto' });
      }, 50);
    }
  }, [activeRoom?.id]); // eslint-disable-line react-hooks/exhaustive-deps

  // Track scroll position to determine if we should auto-scroll
  const handleScroll = useCallback(() => {
    const container = scrollContainerRef.current;
    if (!container) return;

    const { scrollTop, scrollHeight, clientHeight } = container;
    shouldAutoScrollRef.current = scrollHeight - scrollTop - clientHeight < 100;
  }, []);

  // IntersectionObserver for loading older messages
  useEffect(() => {
    if (!activeRoom || !topSentinelRef.current) return;

    const observer = new IntersectionObserver(
      async (entries) => {
        const entry = entries[0];
        if (entry.isIntersecting && !isLoadingMore && roomMessages.length > 0) {
          setIsLoadingMore(true);
          const container = scrollContainerRef.current;
          const prevScrollHeight = container?.scrollHeight || 0;

          const hasMore = await loadMoreMessages(activeRoom.id);

          // Preserve scroll position after loading older messages
          if (hasMore && container) {
            requestAnimationFrame(() => {
              const newScrollHeight = container.scrollHeight;
              container.scrollTop = newScrollHeight - prevScrollHeight;
            });
          }

          setIsLoadingMore(false);
        }
      },
      {
        root: scrollContainerRef.current,
        threshold: 0.1,
      }
    );

    observer.observe(topSentinelRef.current);

    return () => observer.disconnect();
  }, [activeRoom?.id, isLoadingMore, roomMessages.length, loadMoreMessages]); // eslint-disable-line react-hooks/exhaustive-deps

  if (!activeRoom) {
    return (
      <div className="flex-1 flex items-center justify-center bg-[#0f172a]">
        <div className="text-center">
          <div className="inline-flex items-center justify-center w-20 h-20 rounded-full bg-slate-800 mb-4">
            <svg
              className="w-10 h-10 text-slate-600"
              fill="none"
              stroke="currentColor"
              viewBox="0 0 24 24"
            >
              <path
                strokeLinecap="round"
                strokeLinejoin="round"
                strokeWidth={1.5}
                d="M8 12h.01M12 12h.01M16 12h.01M21 12c0 4.418-4.03 8-9 8a9.863 9.863 0 01-4.255-.949L3 20l1.395-3.72C3.512 15.042 3 13.574 3 12c0-4.418 4.03-8 9-8s9 3.582 9 8z"
              />
            </svg>
          </div>
          <h3 className="text-xl font-semibold text-slate-300">Select a conversation</h3>
          <p className="text-sm text-slate-500 mt-2">
            Choose a room or start a new conversation
          </p>
        </div>
      </div>
    );
  }

  // Check if the last message is own for read receipt
  const lastMessage = roomMessages[roomMessages.length - 1];
  const showReadReceipt = lastMessage && lastMessage.sender_id === user?.id;

  return (
    <div
      ref={scrollContainerRef}
      onScroll={handleScroll}
      className="flex-1 overflow-y-auto chat-scrollbar px-4 py-4"
    >
      {/* Top sentinel for loading more */}
      <div ref={topSentinelRef} className="h-1" />

      {isLoadingMore && (
        <div className="flex justify-center py-4">
          <Spinner size="sm" />
        </div>
      )}

      {/* Messages with date separators */}
      {roomMessages.map((message, index) => {
        const prevMessage = index > 0 ? roomMessages[index - 1] : null;
        const showDateSeparator =
          !prevMessage || !isSameDay(prevMessage.created_at, message.created_at);
        const showSender =
          !prevMessage ||
          prevMessage.sender_id !== message.sender_id ||
          showDateSeparator;

        return (
          <div key={message.id}>
            {showDateSeparator && (
              <div className="flex items-center gap-4 py-4">
                <div className="flex-1 h-px bg-slate-700/50" />
                <span className="text-xs text-slate-500 font-medium">
                  {formatDateSeparator(message.created_at)}
                </span>
                <div className="flex-1 h-px bg-slate-700/50" />
              </div>
            )}
            <MessageBubble
              message={message}
              isOwn={message.sender_id === user?.id}
              showSender={showSender}
              isGroupChat={activeRoom.type === 'group'}
            />
          </div>
        );
      })}

      {/* Read receipt */}
      {showReadReceipt && <ReadReceipt isRead={true} />}

      {/* Typing indicator */}
      <TypingIndicator typingUsers={roomTypingUsers} />

      {/* Scroll anchor */}
      <div ref={messagesEndRef} />
    </div>
  );
}
