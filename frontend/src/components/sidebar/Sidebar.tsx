import { useState } from 'react';
import { useAuth } from '../../context/AuthContext';
import { useChat } from '../../context/ChatContext';
import RoomList from './RoomList';
import DMList from './DMList';
import CreateRoomModal from './CreateRoomModal';
import UserSearch from './UserSearch';
import Avatar from '../common/Avatar';
import OnlineIndicator from '../common/OnlineIndicator';

type Tab = 'rooms' | 'dms';

export default function Sidebar() {
  const { user, logout } = useAuth();
  const { isConnected } = useChat();
  const [activeTab, setActiveTab] = useState<Tab>('rooms');
  const [showCreateRoom, setShowCreateRoom] = useState(false);
  const [showUserSearch, setShowUserSearch] = useState(false);

  return (
    <>
      <div className="w-80 h-screen flex flex-col bg-[#1e293b] border-r border-slate-700/50">
        {/* Header */}
        <div className="flex items-center justify-between px-4 py-4 border-b border-slate-700/50">
          <div className="flex items-center gap-3">
            <div className="relative">
              <Avatar username={user?.username || '?'} size="md" />
              <div className="absolute -bottom-0.5 -right-0.5">
                <OnlineIndicator isOnline={isConnected} />
              </div>
            </div>
            <div>
              <p className="text-sm font-semibold text-white">{user?.username}</p>
              <p className="text-xs text-slate-400">
                {isConnected ? 'Online' : 'Connecting...'}
              </p>
            </div>
          </div>
          <button
            onClick={logout}
            className="p-2 text-slate-400 hover:text-red-400 hover:bg-slate-700/50 rounded-lg transition-all"
            title="Logout"
          >
            <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path
                strokeLinecap="round"
                strokeLinejoin="round"
                strokeWidth={2}
                d="M17 16l4-4m0 0l-4-4m4 4H7m6 4v1a3 3 0 01-3 3H6a3 3 0 01-3-3V7a3 3 0 013-3h4a3 3 0 013 3v1"
              />
            </svg>
          </button>
        </div>

        {/* Tab Switcher */}
        <div className="flex border-b border-slate-700/50">
          <button
            onClick={() => setActiveTab('rooms')}
            className={`flex-1 py-3 text-sm font-medium transition-colors relative ${
              activeTab === 'rooms'
                ? 'text-blue-400'
                : 'text-slate-400 hover:text-slate-200'
            }`}
          >
            Rooms
            {activeTab === 'rooms' && (
              <span className="absolute bottom-0 left-1/4 right-1/4 h-0.5 bg-blue-400 rounded-full" />
            )}
          </button>
          <button
            onClick={() => setActiveTab('dms')}
            className={`flex-1 py-3 text-sm font-medium transition-colors relative ${
              activeTab === 'dms'
                ? 'text-blue-400'
                : 'text-slate-400 hover:text-slate-200'
            }`}
          >
            Messages
            {activeTab === 'dms' && (
              <span className="absolute bottom-0 left-1/4 right-1/4 h-0.5 bg-blue-400 rounded-full" />
            )}
          </button>
        </div>

        {/* Action Button */}
        <div className="px-3 py-3">
          {activeTab === 'rooms' ? (
            <button
              onClick={() => setShowCreateRoom(true)}
              className="w-full flex items-center gap-2 px-3 py-2 text-sm text-slate-400 hover:text-white hover:bg-slate-700/50 rounded-lg transition-all"
            >
              <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path
                  strokeLinecap="round"
                  strokeLinejoin="round"
                  strokeWidth={2}
                  d="M12 4v16m8-8H4"
                />
              </svg>
              Create Room
            </button>
          ) : (
            <button
              onClick={() => setShowUserSearch(true)}
              className="w-full flex items-center gap-2 px-3 py-2 text-sm text-slate-400 hover:text-white hover:bg-slate-700/50 rounded-lg transition-all"
            >
              <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path
                  strokeLinecap="round"
                  strokeLinejoin="round"
                  strokeWidth={2}
                  d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z"
                />
              </svg>
              Find Users
            </button>
          )}
        </div>

        {/* Room / DM List */}
        <div className="flex-1 overflow-y-auto chat-scrollbar">
          {activeTab === 'rooms' ? <RoomList /> : <DMList />}
        </div>
      </div>

      <CreateRoomModal isOpen={showCreateRoom} onClose={() => setShowCreateRoom(false)} />
      <UserSearch isOpen={showUserSearch} onClose={() => setShowUserSearch(false)} />
    </>
  );
}
