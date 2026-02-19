import { useState, useEffect, useRef, useCallback } from 'react';
import { searchUsers, startDM } from '../../api/rooms';
import { useChat } from '../../context/ChatContext';
import Avatar from '../common/Avatar';
import Spinner from '../common/Spinner';

interface UserSearchProps {
  isOpen: boolean;
  onClose: () => void;
}

interface SearchUser {
  id: string;
  username: string;
  email: string;
}

export default function UserSearch({ isOpen, onClose }: UserSearchProps) {
  const [query, setQuery] = useState('');
  const [results, setResults] = useState<SearchUser[]>([]);
  const [isSearching, setIsSearching] = useState(false);
  const [isStartingDM, setIsStartingDM] = useState<string | null>(null);
  const debounceRef = useRef<ReturnType<typeof setTimeout> | null>(null);
  const inputRef = useRef<HTMLInputElement>(null);
  const { setActiveRoom } = useChat();

  useEffect(() => {
    if (isOpen && inputRef.current) {
      inputRef.current.focus();
    }
    if (!isOpen) {
      setQuery('');
      setResults([]);
    }
  }, [isOpen]);

  const performSearch = useCallback(async (searchQuery: string) => {
    if (searchQuery.trim().length < 1) {
      setResults([]);
      return;
    }

    setIsSearching(true);
    try {
      const response = await searchUsers(searchQuery.trim());
      setResults(response.users || []);
    } catch {
      setResults([]);
    } finally {
      setIsSearching(false);
    }
  }, []);

  const handleQueryChange = (value: string) => {
    setQuery(value);

    if (debounceRef.current) {
      clearTimeout(debounceRef.current);
    }

    debounceRef.current = setTimeout(() => {
      performSearch(value);
    }, 300);
  };

  const handleStartDM = async (userId: string) => {
    setIsStartingDM(userId);
    try {
      const response = await startDM(userId);
      setActiveRoom(response.room);
      onClose();
    } catch (err) {
      console.error('Failed to start DM:', err);
    } finally {
      setIsStartingDM(null);
    }
  };

  const handleBackdropClick = (e: React.MouseEvent) => {
    if (e.target === e.currentTarget) {
      onClose();
    }
  };

  if (!isOpen) return null;

  return (
    <div
      className="fixed inset-0 z-50 flex items-start justify-center pt-20 bg-black/50 modal-backdrop"
      onClick={handleBackdropClick}
    >
      <div className="w-full max-w-md mx-4 bg-[#1e293b] rounded-2xl shadow-2xl border border-slate-700/50 overflow-hidden">
        <div className="px-4 py-3 border-b border-slate-700">
          <div className="flex items-center gap-3">
            <svg
              className="w-5 h-5 text-slate-400 flex-shrink-0"
              fill="none"
              stroke="currentColor"
              viewBox="0 0 24 24"
            >
              <path
                strokeLinecap="round"
                strokeLinejoin="round"
                strokeWidth={2}
                d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z"
              />
            </svg>
            <input
              ref={inputRef}
              type="text"
              value={query}
              onChange={(e) => handleQueryChange(e.target.value)}
              placeholder="Search users by username..."
              className="flex-1 bg-transparent text-white placeholder-slate-500 focus:outline-none text-sm"
            />
            <button
              onClick={onClose}
              className="text-slate-400 hover:text-white transition-colors p-1"
            >
              <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path
                  strokeLinecap="round"
                  strokeLinejoin="round"
                  strokeWidth={2}
                  d="M6 18L18 6M6 6l12 12"
                />
              </svg>
            </button>
          </div>
        </div>

        <div className="max-h-80 overflow-y-auto chat-scrollbar">
          {isSearching && (
            <div className="flex items-center justify-center py-8">
              <Spinner size="sm" />
            </div>
          )}

          {!isSearching && query && results.length === 0 && (
            <div className="py-8 text-center">
              <p className="text-slate-500 text-sm">No users found</p>
            </div>
          )}

          {!isSearching && results.length > 0 && (
            <div className="py-2">
              {results.map((u) => (
                <button
                  key={u.id}
                  onClick={() => handleStartDM(u.id)}
                  disabled={isStartingDM === u.id}
                  className="w-full flex items-center gap-3 px-4 py-3 hover:bg-slate-700/50 transition-colors text-left disabled:opacity-50"
                >
                  <Avatar username={u.username} size="sm" />
                  <div className="flex-1 min-w-0">
                    <p className="text-sm font-medium text-white truncate">{u.username}</p>
                    <p className="text-xs text-slate-400 truncate">{u.email}</p>
                  </div>
                  {isStartingDM === u.id ? (
                    <Spinner size="sm" />
                  ) : (
                    <svg
                      className="w-4 h-4 text-slate-400"
                      fill="none"
                      stroke="currentColor"
                      viewBox="0 0 24 24"
                    >
                      <path
                        strokeLinecap="round"
                        strokeLinejoin="round"
                        strokeWidth={2}
                        d="M8 12h.01M12 12h.01M16 12h.01M21 12c0 4.418-4.03 8-9 8a9.863 9.863 0 01-4.255-.949L3 20l1.395-3.72C3.512 15.042 3 13.574 3 12c0-4.418 4.03-8 9-8s9 3.582 9 8z"
                      />
                    </svg>
                  )}
                </button>
              ))}
            </div>
          )}

          {!query && (
            <div className="py-8 text-center">
              <p className="text-slate-500 text-sm">Type to search for users</p>
            </div>
          )}
        </div>
      </div>
    </div>
  );
}
