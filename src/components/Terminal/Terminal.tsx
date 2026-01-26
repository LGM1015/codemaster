import { useEffect, useRef } from 'react';
import { Terminal as XTerm } from '@xterm/xterm';
import { FitAddon } from '@xterm/addon-fit';
import '@xterm/xterm/css/xterm.css';
import './Terminal.css';

export function Terminal() {
  const terminalRef = useRef<HTMLDivElement>(null);
  const xtermRef = useRef<XTerm | null>(null);

  useEffect(() => {
    if (!terminalRef.current) return;

    const term = new XTerm({
      theme: {
        background: '#0f3460',
        foreground: '#ffffff',
      },
      cursorBlink: true,
      fontFamily: 'Consolas, monospace',
      fontSize: 14,
    });
    
    const fitAddon = new FitAddon();
    term.loadAddon(fitAddon);
    
    term.open(terminalRef.current);
    fitAddon.fit();
    
    term.writeln('CodeMaster Terminal [Version 0.1.0]');
    term.writeln('(c) 2026 CodeMaster Contributors. All rights reserved.');
    term.writeln('');
    term.write('PS E:\\codemaster> ');

    term.onData(data => {
        // Simple echo for now
        if (data === '\r') {
            term.writeln('');
            term.write('PS E:\\codemaster> ');
        } else if (data === '\u007f') { // Backspace
            term.write('\b \b');
        } else {
            term.write(data);
        }
    });

    xtermRef.current = term;

    const handleResize = () => fitAddon.fit();
    window.addEventListener('resize', handleResize);

    return () => {
      term.dispose();
      window.removeEventListener('resize', handleResize);
    };
  }, []);

  return <div className="terminal-container" ref={terminalRef} />;
}
