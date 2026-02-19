import type { Room } from '../../types';
import { useChat } from '../../context/ChatContext';
import DMItem from './DMItem';

export default function DMList() {
  const { rooms, activeRoom, setActiveRoom } = useChat();

  const dmRooms = rooms
    .filter((r) => r.type === 'dm')
    .sort((a, b) => {
      const aTime = a.last_message_at ? new Date(a.last_message_at).getTime() : 0;
      const bTime = b.last_message_at ? new Date(b.last_message_at).getTime() : 0;
      return bTime - aTime;
    });

  if (dmRooms.length === 0) {
    return (
      <div className="px-4 py-8 text-center">
        <p className="text-slate-500 text-sm">No conversations yet</p>
        <p className="text-slate-600 text-xs mt-1">Search for users to start chatting</p>
      </div>
    );
  }

  return (
    <div className="space-y-1 px-2">
      {dmRooms.map((room: Room) => (
        <DMItem
          key={room.id}
          room={room}
          isActive={activeRoom?.id === room.id}
          onClick={() => setActiveRoom(room)}
        />
      ))}
    </div>
  );
}
