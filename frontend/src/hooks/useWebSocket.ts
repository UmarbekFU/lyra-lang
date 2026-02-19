import { useRef, useState, useCallback, useEffect } from 'react';
import type { WSMessage } from '../types';

type MessageHandler = (payload: any) => void;

interface UseWebSocketReturn {
  connect: (token: string) => void;
  disconnect: () => void;
  send: (type: string, payload?: unknown) => void;
  on: (type: string, callback: MessageHandler) => () => void;
  isConnected: boolean;
}

export function useWebSocket(): UseWebSocketReturn {
  const [isConnected, setIsConnected] = useState(false);
  const wsRef = useRef<WebSocket | null>(null);
  const listenersRef = useRef<Map<string, Set<MessageHandler>>>(new Map());
  const reconnectAttemptRef = useRef(0);
  const reconnectTimerRef = useRef<ReturnType<typeof setTimeout> | null>(null);
  const tokenRef = useRef<string | null>(null);
  const intentionalCloseRef = useRef(false);
  const pingIntervalRef = useRef<ReturnType<typeof setInterval> | null>(null);

  const MAX_RECONNECT_ATTEMPTS = 10;
  const MAX_RECONNECT_DELAY = 30000;

  const clearReconnectTimer = useCallback(() => {
    if (reconnectTimerRef.current) {
      clearTimeout(reconnectTimerRef.current);
      reconnectTimerRef.current = null;
    }
  }, []);

  const clearPingInterval = useCallback(() => {
    if (pingIntervalRef.current) {
      clearInterval(pingIntervalRef.current);
      pingIntervalRef.current = null;
    }
  }, []);

  const emit = useCallback((type: string, payload: unknown) => {
    const handlers = listenersRef.current.get(type);
    if (handlers) {
      handlers.forEach((handler) => handler(payload));
    }
  }, []);

  const connectWs = useCallback((token: string) => {
    intentionalCloseRef.current = false;
    tokenRef.current = token;

    if (wsRef.current) {
      wsRef.current.close();
    }

    const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
    const wsUrl = `${protocol}//${window.location.host}/ws?token=${encodeURIComponent(token)}`;
    const ws = new WebSocket(wsUrl);

    ws.onopen = () => {
      setIsConnected(true);
      reconnectAttemptRef.current = 0;

      clearPingInterval();
      pingIntervalRef.current = setInterval(() => {
        if (ws.readyState === WebSocket.OPEN) {
          ws.send(JSON.stringify({ type: 'ping' }));
        }
      }, 30000);
    };

    ws.onmessage = (event) => {
      try {
        const msg: WSMessage = JSON.parse(event.data);
        emit(msg.type, msg.payload);
      } catch {
        // Ignore malformed messages
      }
    };

    ws.onclose = () => {
      setIsConnected(false);
      clearPingInterval();

      if (!intentionalCloseRef.current && tokenRef.current) {
        if (reconnectAttemptRef.current < MAX_RECONNECT_ATTEMPTS) {
          const delay = Math.min(
            1000 * Math.pow(2, reconnectAttemptRef.current),
            MAX_RECONNECT_DELAY
          );
          reconnectAttemptRef.current++;

          clearReconnectTimer();
          reconnectTimerRef.current = setTimeout(() => {
            if (tokenRef.current) {
              connectWs(tokenRef.current);
            }
          }, delay);
        }
      }
    };

    ws.onerror = () => {
      // onclose will be called after onerror
    };

    wsRef.current = ws;
  }, [emit, clearReconnectTimer, clearPingInterval]);

  const disconnect = useCallback(() => {
    intentionalCloseRef.current = true;
    tokenRef.current = null;
    clearReconnectTimer();
    clearPingInterval();
    if (wsRef.current) {
      wsRef.current.close();
      wsRef.current = null;
    }
    setIsConnected(false);
  }, [clearReconnectTimer, clearPingInterval]);

  const send = useCallback((type: string, payload?: unknown) => {
    if (wsRef.current && wsRef.current.readyState === WebSocket.OPEN) {
      wsRef.current.send(JSON.stringify({ type, payload }));
    }
  }, []);

  const on = useCallback((type: string, callback: MessageHandler): (() => void) => {
    if (!listenersRef.current.has(type)) {
      listenersRef.current.set(type, new Set());
    }
    listenersRef.current.get(type)!.add(callback);

    return () => {
      const handlers = listenersRef.current.get(type);
      if (handlers) {
        handlers.delete(callback);
        if (handlers.size === 0) {
          listenersRef.current.delete(type);
        }
      }
    };
  }, []);

  useEffect(() => {
    return () => {
      intentionalCloseRef.current = true;
      clearReconnectTimer();
      clearPingInterval();
      if (wsRef.current) {
        wsRef.current.close();
      }
    };
  }, [clearReconnectTimer, clearPingInterval]);

  return {
    connect: connectWs,
    disconnect,
    send,
    on,
    isConnected,
  };
}
