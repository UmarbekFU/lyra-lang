interface AvatarProps {
  username: string;
  size?: 'sm' | 'md' | 'lg';
  className?: string;
}

const AVATAR_COLORS = [
  'bg-blue-500',
  'bg-emerald-500',
  'bg-violet-500',
  'bg-amber-500',
  'bg-rose-500',
  'bg-cyan-500',
  'bg-pink-500',
  'bg-teal-500',
  'bg-indigo-500',
  'bg-orange-500',
  'bg-lime-500',
  'bg-fuchsia-500',
];

function hashString(str: string): number {
  let hash = 0;
  for (let i = 0; i < str.length; i++) {
    const char = str.charCodeAt(i);
    hash = (hash << 5) - hash + char;
    hash |= 0;
  }
  return Math.abs(hash);
}

const sizeClasses = {
  sm: 'w-8 h-8 text-xs',
  md: 'w-10 h-10 text-sm',
  lg: 'w-12 h-12 text-base',
};

export default function Avatar({ username, size = 'md', className = '' }: AvatarProps) {
  const colorIndex = hashString(username) % AVATAR_COLORS.length;
  const bgColor = AVATAR_COLORS[colorIndex];
  const letter = username.charAt(0).toUpperCase();

  return (
    <div
      className={`${sizeClasses[size]} ${bgColor} rounded-full flex items-center justify-center font-semibold text-white select-none flex-shrink-0 ${className}`}
    >
      {letter}
    </div>
  );
}
