import { useEffect, useRef, useImperativeHandle, forwardRef } from 'react';
import { Terminal as XTerm } from '@xterm/xterm';
import { FitAddon } from '@xterm/addon-fit';
import { listen } from '@tauri-apps/api/event';
import { AgentEvent } from '../../types';
import '@xterm/xterm/css/xterm.css';
import './Terminal.css';

export interface TerminalHandle {
  write: (text: string) => void;
  writeln: (text: string) => void;
  clear: () => void;
}

export const Terminal = forwardRef<TerminalHandle>((_, ref) => {
  const terminalRef = useRef<HTMLDivElement>(null);
  const xtermRef = useRef<XTerm | null>(null);

  useImperativeHandle(ref, () => ({
    write: (text: string) => xtermRef.current?.write(text),
    writeln: (text: string) => xtermRef.current?.writeln(text),
    clear: () => xtermRef.current?.clear(),
  }));

  useEffect(() => {
    if (!terminalRef.current) return;

    const term = new XTerm({
      theme: {
        background: '#0a1929',
        foreground: '#b2c5d6',
        cyan: '#22d3ee',
        green: '#10b981',
        yellow: '#f59e0b',
        red: '#f43f5e',
        magenta: '#a855f7',
        cursor: '#22d3ee',
        cursorAccent: '#0a1929',
      },
      cursorBlink: true,
      fontFamily: 'Consolas, "Courier New", monospace',
      fontSize: 13,
      lineHeight: 1.2,
      scrollback: 1000,
    });
    
    const fitAddon = new FitAddon();
    term.loadAddon(fitAddon);
    
    term.open(terminalRef.current);
    fitAddon.fit();
    
    // Welcome message
    term.writeln('\x1b[36m╔══════════════════════════════════════════╗\x1b[0m');
    term.writeln('\x1b[36m║\x1b[0m  \x1b[1;35mCodeMaster Terminal\x1b[0m v0.1.0              \x1b[36m║\x1b[0m');
    term.writeln('\x1b[36m║\x1b[0m  Agent 命令执行输出将显示在此处         \x1b[36m║\x1b[0m');
    term.writeln('\x1b[36m╚══════════════════════════════════════════╝\x1b[0m');
    term.writeln('');

    xtermRef.current = term;

    // Listen for bash execution events from agent
    const unlistenBash = listen<AgentEvent>('agent-event', (event) => {
      const payload = event.payload;
      
      if (payload.type === 'ToolCall') {
        const content = payload.content;
        if (content.name === 'bash') {
          try {
            const args = JSON.parse(content.args);
            term.writeln('');
            term.writeln(`\x1b[33m┌─ 执行命令 ─────────────────────────────\x1b[0m`);
            term.writeln(`\x1b[33m│\x1b[0m \x1b[1;36m$ ${args.command}\x1b[0m`);
            if (args.workdir) {
              term.writeln(`\x1b[33m│\x1b[0m \x1b[90m工作目录: ${args.workdir}\x1b[0m`);
            }
            term.writeln(`\x1b[33m└────────────────────────────────────────\x1b[0m`);
          } catch (e) {
            // Ignore parse errors
          }
        }
      }
      
      if (payload.type === 'ToolResult') {
        const content = payload.content;
        if (content.name === 'bash') {
          // Format output
          const lines = content.result.split('\n');
          const isError = content.result.startsWith('Exit Code:') || content.result.includes('Error:');
          
          term.writeln('');
          if (isError) {
            term.writeln(`\x1b[31m┌─ 执行失败 ─────────────────────────────\x1b[0m`);
          } else {
            term.writeln(`\x1b[32m┌─ 执行成功 ─────────────────────────────\x1b[0m`);
          }
          
          // Limit output lines
          const maxLines = 50;
          const displayLines = lines.slice(0, maxLines);
          displayLines.forEach(line => {
            const color = isError ? '\x1b[31m' : '\x1b[0m';
            term.writeln(`${color}│\x1b[0m ${line}`);
          });
          
          if (lines.length > maxLines) {
            term.writeln(`\x1b[90m│ ... (省略 ${lines.length - maxLines} 行)\x1b[0m`);
          }
          
          const borderColor = isError ? '\x1b[31m' : '\x1b[32m';
          term.writeln(`${borderColor}└────────────────────────────────────────\x1b[0m`);
        }
      }
    });

    const handleResize = () => fitAddon.fit();
    window.addEventListener('resize', handleResize);

    // Also fit when the terminal panel is toggled
    const resizeObserver = new ResizeObserver(() => {
      setTimeout(() => fitAddon.fit(), 0);
    });
    resizeObserver.observe(terminalRef.current);

    return () => {
      term.dispose();
      window.removeEventListener('resize', handleResize);
      unlistenBash.then(f => f());
      resizeObserver.disconnect();
    };
  }, []);

  return <div className="terminal-container" ref={terminalRef} />;
});
