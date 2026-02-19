import { useAuth } from '../../context/AuthContext';
import { useChat } from '../../context/ChatContext';
import Sidebar from '../sidebar/Sidebar';
import MessageList from './MessageList';
import MessageInput from './MessageInput';
import Avatar from '../common/Avatar';
import OnlineIndicator from '../common/OnlineIndicator';

function getOtherUsername(roomName: string, currentUsername: string): string {
  const parts = roomName.split(':');
  if (parts.length === 2) {
    return parts[0] === currentUsername ? parts[1] : parts[0];
  }
  return roomName;
}

export default function ChatLayout() {
  const { user } = useAuth();
  const { activeRoom, onlineUsers } = useChat();

  const isDM = activeRoom?.type === 'dm';
  const displayName = isDM
    ? getOtherUsername(activeRoom?.name || '', user?.username || '')
    : activeRoom?.name;

  const otherUser = isDM ? activeRoom?.members?.find((m) => m.id !== user?.id) : null;
  const isOtherOnline = otherUser ? onlineUsers.has(otherUser.id) : false;

  return (
    <div className="flex h-screen bg-[#0f172a]">
      {/* Sidebar */}
      <Sidebar />

      {/* Main Chat Area */}
      <div className="flex-1 flex flex-col min-w-0">
        {/* Chat Header */}
        {activeRoom && (
          <div className="flex items-center gap-3 px-6 py-4 bg-[#1e293b] border-b border-slate-700/50">
            <div className="relative">
              <Avatar username={displayName || '?'} size="md" />
              {isDM && (
                <div className="absolute -bottom-0.5 -right-0.5">
                  <OnlineIndicator isOnline={isOtherOnline} />
                </div>
              )}
            </div>
            <div>
              <h2 className="text-base font-semibold text-white">
                {isDM ? displayName : `# ${displayName}`}
              </h2>
              <p className="text-xs text-slate-400">
                {isDM
                  ? isOtherOnline
                    ? 'Online'
                    : 'Offline'
                  : activeRoom.members
                  ? `${activeRoom.members.length} members`
                  : 'Room'}
              </p>
            </div>
          </div>
        )}

        {/* Messages */}
        <MessageList />

        {/* Input */}
        <MessageInput />
      </div>
    </div>
  );
}
