import { useState, useEffect, useMemo } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { useTranslation } from 'react-i18next';
import { Layout } from './components/Layout';
import { Terminal } from './components/Terminal/Terminal';
import { ChatInput } from './components/Chat/ChatInput';
import { MessageList } from './components/Chat/MessageList';
import { Settings } from './components/Settings/Settings';
import { SessionList } from './components/Session/SessionList';
import { Message, AgentEvent } from './types';
import { useChatStore } from './store/chatStore';
import './App.css';

interface Session {
  id: string;
  title: string;
  created_at: number;
  updated_at: number;
}

function App() {
  const { t } = useTranslation();
  const [showSettings, setShowSettings] = useState(false);
  
  const { 
    messages, 
    loading, 
    streamingContent, 
    currentSessionId, 
    sessionRefreshTrigger,
    setMessages,
    addMessage,
    setLoading,
    setStreamingContent,
    appendStreamingContent,
    setCurrentSessionId,
    triggerSessionRefresh,
    loadSession,
    resetSession
  } = useChatStore();

  // Load initial welcome message
  useEffect(() => {
    if (!currentSessionId && messages.length === 0) {
      setMessages([{ 
        role: 'assistant', 
        content: t('welcome.description')
      }]);
    }
  }, [t, currentSessionId, messages.length, setMessages]);

  // Event listener for agent events
  useEffect(() => {
    const unlisten = listen<AgentEvent>('agent-event', (event) => {
      const payload = event.payload;
      console.log('Event received:', payload);
      
      if (payload.type === 'Thinking') {
          setStreamingContent('思考中...');
      } else if (payload.type === 'StreamChunk') {
          // If content is "Thinking...", replace it, otherwise append
          if (useChatStore.getState().streamingContent === '思考中...') {
             setStreamingContent(payload.content as string);
          } else {
             appendStreamingContent(payload.content as string);
          }
      } else if (payload.type === 'StreamEnd') {
          setStreamingContent('');
      } else if (payload.type === 'NewMessage') {
          const msg = payload.content as Message;
          addMessage(msg);
          // Save message to DB
          const sessionId = useChatStore.getState().currentSessionId;
          if (sessionId) {
            saveMessageToDb(sessionId, msg);
          }
      } else if (payload.type === 'ToolResult') {
           const content = payload.content as { name: string, result: string, id: string };
           const toolMsg: Message = {
              role: 'tool',
              name: content.name,
              content: content.result,
              tool_call_id: content.id
           };
           addMessage(toolMsg);
           const sessionId = useChatStore.getState().currentSessionId;
           if (sessionId) {
             saveMessageToDb(sessionId, toolMsg);
           }
      } else if (payload.type === 'Error') {
           addMessage({ role: 'assistant', content: `❌ Error: ${payload.content}` });
           setLoading(false);
           setStreamingContent('');
      } else if (payload.type === 'Done') {
           setLoading(false);
           setStreamingContent('');
      }
    });

    return () => {
      unlisten.then(f => f());
    };
  }, []); // Empty dependency array as we use useChatStore.getState() or store actions

  const saveMessageToDb = async (sessionId: string, msg: Message) => {
    try {
      await invoke('save_message', {
        sessionId,
        role: msg.role,
        content: msg.content || null,
        toolCalls: msg.tool_calls ? JSON.stringify(msg.tool_calls) : null,
        toolCallId: msg.tool_call_id || null,
        name: msg.name || null,
      });
    } catch (e) {
      console.error('Failed to save message:', e);
    }
  };

  const handleSend = async (text: string) => {
    let sessionId = currentSessionId;
    
    // Create session if needed
    if (!sessionId) {
      try {
        const title = text.slice(0, 30) + (text.length > 30 ? '...' : '');
        const session = await invoke<Session>('create_session', { title });
        sessionId = session.id;
        setCurrentSessionId(sessionId);
        triggerSessionRefresh();
      } catch (e) {
        console.error('Failed to create session:', e);
        return;
      }
    }

    const newMsg: Message = { role: 'user', content: text };
    addMessage(newMsg);
    setLoading(true);

    // Save user message
    await saveMessageToDb(sessionId, newMsg);

    try {
      // Use current messages + new message for history
      // Note: messages here is from closure, so it doesn't have newMsg yet
      await invoke('send_message', { message: text, history: [...messages, newMsg] });
    } catch (e) {
      addMessage({ role: 'assistant', content: `Error sending message: ${e}` });
      setLoading(false);
    }
  };

  const handleSelectSession = async (id: string) => {
    await loadSession(id);
  };

  const handleNewSession = () => {
    resetSession();
    // Welcome message will be added by the useEffect
  };

  const Sidebar = useMemo(() => (
    <SessionList
      currentSessionId={currentSessionId}
      onSelectSession={handleSelectSession}
      onNewSession={handleNewSession}
      refreshTrigger={sessionRefreshTrigger}
    />
  ), [currentSessionId, sessionRefreshTrigger, handleSelectSession, handleNewSession]);

  return (
    <>
      <Layout 
        sidebar={Sidebar}
        terminal={<Terminal />}
        onSettingsClick={() => setShowSettings(true)}
        content={
          <>
            <MessageList messages={messages} streamingContent={streamingContent} />
            <ChatInput onSend={handleSend} disabled={loading} />
          </>
        }
      />
      {showSettings && <Settings onClose={() => setShowSettings(false)} />}
    </>
  );
}

export default App;
