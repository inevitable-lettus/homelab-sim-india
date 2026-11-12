// The terminal screen itself. Loaded only when the learner asks for it, so
// the terminal emulator is not part of the first download.
import { FitAddon } from '@xterm/addon-fit';
import { Terminal } from '@xterm/xterm';
import '@xterm/xterm/css/xterm.css';
import { useEffect, useRef } from 'react';
import type { TerminalHost } from '../lib/terminal';

export default function TerminalView({ host }: { host: TerminalHost }) {
  const box = useRef<HTMLDivElement>(null);
  useEffect(() => {
    const element = box.current;
    if (!element) return;
    const style = getComputedStyle(element);
    const term = new Terminal({
      cursorBlink: false,
      convertEol: false,
      fontFamily: style.fontFamily,
      fontSize: 13,
      screenReaderMode: true,
      scrollback: 1000,
      theme: { background: '#16181d', foreground: '#e9ebef', cursor: '#e9ebef' },
    });
    const fit = new FitAddon();
    term.loadAddon(fit);
    term.open(element);
    fit.fit();
    host.onOutput((data) => term.write(data));
    const input = term.onData((data) => host.writeInput(data));
    // Tell the guest how big the screen is, then show a clean prompt.
    host.writeInput(` stty cols ${term.cols} rows ${term.rows}; clear\n`);
    return () => {
      input.dispose();
      term.dispose();
    };
  }, [host]);
  return <div ref={box} className="terminal__screen mono" data-testid="terminal-screen" />;
}
