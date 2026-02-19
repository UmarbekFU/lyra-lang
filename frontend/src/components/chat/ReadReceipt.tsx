interface ReadReceiptProps {
  isRead: boolean;
}

export default function ReadReceipt({ isRead }: ReadReceiptProps) {
  if (!isRead) return null;

  return (
    <div className="flex justify-end px-4 py-0.5">
      <div className="flex items-center gap-1 text-blue-400">
        <svg className="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path
            strokeLinecap="round"
            strokeLinejoin="round"
            strokeWidth={2.5}
            d="M5 13l4 4L19 7"
          />
        </svg>
        <svg
          className="w-3.5 h-3.5 -ml-2"
          fill="none"
          stroke="currentColor"
          viewBox="0 0 24 24"
        >
          <path
            strokeLinecap="round"
            strokeLinejoin="round"
            strokeWidth={2.5}
            d="M5 13l4 4L19 7"
          />
        </svg>
        <span className="text-[10px] text-slate-500 ml-0.5">Read</span>
      </div>
    </div>
  );
}
