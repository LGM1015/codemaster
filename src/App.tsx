import { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { Layout } from './components/Layout';
import { Terminal } from './components/Terminal/Terminal';
import { ChatInput } from './components/Chat/ChatInput';
import { MessageList } from './components/Chat/MessageList';
import { Settings } from './components/Settings/Settings';
import { Message, AgentEvent } from './types';
import './App.css';

function App() {
  const [showSettings, setShowSettings] = useState(false);
  const [messages, setMessages] = useState<Message[]>([]);
  const [loading, setLoading] = useState(false);
  const [streamingContent] = useState('');

  useEffect(() => {
    setMessages([{ role: 'assistant', content: 'Welcome to CodeMaster! I am powered by DeepSeek. Please set your API Key in settings.' }]);
  }, []);

  useEffect(() => {
    const unlisten = listen<AgentEvent>('agent-event', (event) => {
      const payload = event.payload;
      console.log('Event received:', payload);
      
      // Rust Serde serialization of enum:
      // #[serde(tag = "type", content = "content")]
      // Payload structure: { type: "Message", content: "..." }

      if (payload.type === 'Thinking') {
          // Ignore thinking for now or show spinner
      } else if (payload.type === 'Message') {
          setMessages(prev => [...prev, { role: 'assistant', content: payload.content as string }]);
      } else if (payload.type === 'ToolCall') {
          const content = payload.content as { name: string, args: string };
          setMessages(prev => [...prev, { 
             role: 'assistant', 
             tool_calls: [{ 
                 id: 'call_' + Date.now(), 
                 type: 'function', 
                 function: { name: content.name, arguments: content.args }
             }]
          }]);
      } else if (payload.type === 'ToolResult') {
           const content = payload.content as { name: string, result: string };
           setMessages(prev => [...prev, {
              role: 'tool',
              name: content.name,
              content: content.result
           }]);
      } else if (payload.type === 'Error') {
           setMessages(prev => [...prev, { role: 'assistant', content: `❌ Error: ${payload.content}` }]);
           setLoading(false);
      } else if (payload.type === 'Done') {
           setLoading(false);
      }
    });

    return () => {
      unlisten.then(f => f());
    };
  }, []);

  const handleSend = async (text: string) => {
    const newMsg: Message = { role: 'user', content: text };
    setMessages(prev => [...prev, newMsg]);
    setLoading(true);

    try {
      await invoke('send_message', { message: text, history: messages });
    } catch (e) {
      setMessages(prev => [...prev, { role: 'assistant', content: `Error sending message: ${e}` }]);
      setLoading(false);
    }
  };

  const Sidebar = (
    <div style={{padding: '1rem', color: '#888'}}>
      <h3>Sessions</h3>
      <p style={{fontSize: '0.8rem', marginTop: '1rem'}}>
        Local sessions are stored in SQLite.<br/>
        (UI Management coming soon)
      </p>
    </div>
  );

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
