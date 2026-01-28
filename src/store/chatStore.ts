import { create } from 'zustand';
import { invoke } from '@tauri-apps/api/core';
import { Message } from '../types';

interface ChatState {
  messages: Message[];
  loading: boolean;
  streamingContent: string;
  currentSessionId: string | null;
  sessionRefreshTrigger: number; // Increment to force sidebar refresh

  // Sync actions
  setMessages: (messages: Message[]) => void;
  addMessage: (message: Message) => void;
  setLoading: (loading: boolean) => void;
  setStreamingContent: (content: string) => void;
  appendStreamingContent: (content: string) => void;
  setCurrentSessionId: (id: string | null) => void;
  triggerSessionRefresh: () => void;
  
  // Async actions
  loadSession: (sessionId: string) => Promise<void>;
  resetSession: () => void;
}

export const useChatStore = create<ChatState>((set) => ({
  messages: [],
  loading: false,
  streamingContent: '',
  currentSessionId: null,
  sessionRefreshTrigger: 0,

  setMessages: (messages) => set({ messages }),
  addMessage: (message) => set((state) => ({ messages: [...state.messages, message] })),
  setLoading: (loading) => set({ loading }),
  setStreamingContent: (content) => set({ streamingContent: content }),
  appendStreamingContent: (content) => set((state) => ({ streamingContent: state.streamingContent + content })),
  setCurrentSessionId: (id) => set({ currentSessionId: id }),
  triggerSessionRefresh: () => set((state) => ({ sessionRefreshTrigger: state.sessionRefreshTrigger + 1 })),

  loadSession: async (sessionId: string) => {
    try {
      set({ loading: true, streamingContent: '' });
      const sessionMessages = await invoke<Message[]>('get_session_messages', { sessionId });
      set({ 
        messages: sessionMessages, 
        currentSessionId: sessionId, 
        loading: false 
      });
    } catch (e) {
      console.error('Failed to load session:', e);
      set({ loading: false });
    }
  },

  resetSession: () => {
    set({ 
      currentSessionId: null, 
      messages: [], 
      streamingContent: '', 
      loading: false 
    });
  }
}));
