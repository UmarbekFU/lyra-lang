interface OnlineIndicatorProps {
  isOnline: boolean;
  className?: string;
}

export default function OnlineIndicator({ isOnline, className = '' }: OnlineIndicatorProps) {
  return (
    <span
      className={`inline-block w-2.5 h-2.5 rounded-full flex-shrink-0 ${
        isOnline ? 'bg-green-500' : 'bg-slate-500'
      } ${className}`}
    />
  );
}
