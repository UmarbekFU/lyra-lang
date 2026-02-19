import type { Room } from '../../types';
import { useChat } from '../../context/ChatContext';
import RoomItem from './RoomItem';

export default function RoomList() {
  const { rooms, activeRoom, setActiveRoom } = useChat();

  const groupRooms = rooms
    .filter((r) => r.type === 'group')
    .sort((a, b) => {
      const aTime = a.last_message_at ? new Date(a.last_message_at).getTime() : 0;
      const bTime = b.last_message_at ? new Date(b.last_message_at).getTime() : 0;
      return bTime - aTime;
    });

  if (groupRooms.length === 0) {
    return (
      <div className="px-4 py-8 text-center">
        <p className="text-slate-500 text-sm">No rooms yet</p>
        <p className="text-slate-600 text-xs mt-1">Create a room to get started</p>
      </div>
    );
  }

  return (
    <div className="space-y-1 px-2">
      {groupRooms.map((room: Room) => (
        <RoomItem
          key={room.id}
          room={room}
          isActive={activeRoom?.id === room.id}
          onClick={() => setActiveRoom(room)}
        />
      ))}
    </div>
  );
}
