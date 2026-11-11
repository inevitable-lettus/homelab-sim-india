// What the core knows about one box on the board, and the controls that
// belong to it.
import { useEffect, useRef } from 'react';
import { StatusMark, verdictWord } from '../components/Status';
import { CloseIcon, NodeIcon } from '../components/icons';
import { distance, rate } from '../lib/format';
import type { Addr, Bundle, Id, Mission, NodeFacts, World } from '../lib/types';
import { Controls } from './Setup';

interface Props {
  nodeId: Id;
  mission: Mission;
  world: World;
  facts: NodeFacts[];
  bundle: Bundle;
  onChange: (world: World) => void;
  onClose: () => void;
}

const show = (addr: Addr | undefined) => (addr === null || addr === undefined ? 'none' : addr);

const KIND: Record<string, string> = {
  host: 'A device with an address',
  router: 'Home router. Shares one WAN address between everything at home',
  isp: 'Internet provider',
  internet: 'Everything between the providers',
};

export function Inspector({ nodeId, mission, world, facts, bundle, onChange, onClose }: Props) {
  const node = world.nodes.find((n) => n.id === nodeId);
  const fact = facts.find((f) => f.id === nodeId);
  const close = useRef<HTMLButtonElement>(null);

  useEffect(() => {
    close.current?.focus();
    const onKey = (event: KeyboardEvent) => {
      if (event.key === 'Escape') onClose();
    };
    addEventListener('keydown', onKey);
    return () => removeEventListener('keydown', onKey);
  }, [nodeId, onClose]);

  if (!node || !fact) return null;
  const links = world.links.filter((l) => l.a === nodeId || l.b === nodeId);
  const other = (id: Id) => world.nodes.find((n) => n.id === id)?.label ?? id;
  const profile = node.kind === 'isp' && node.profile ? bundle.isps[node.profile] : undefined;

  return (
    <aside className="inspector" aria-label={`About ${node.label}`} data-testid="inspector">
      <header className="inspector__head">
        <span className="inspector__icon">
          <NodeIcon kind={node.kind === 'isp' && node.id === 'mobile' ? 'mobile' : node.kind} hint={node.kind === 'host' ? node.icon : undefined} />
        </span>
        <div>
          <h3>{node.label}</h3>
          <p>{KIND[node.kind]}</p>
        </div>
        <button ref={close} type="button" className="icon-btn" onClick={onClose} aria-label="Close">
          <CloseIcon />
        </button>
      </header>

      <Controls mission={mission} world={world} facts={facts} onChange={onChange} node={nodeId} />

      <dl className="facts facts--list">
        {node.kind === 'host' && (
          <>
            <div>
              <dt>Its own address</dt>
              <dd>{show(fact.ipv4)}</dd>
            </div>
            <div>
              <dt>What the internet sees</dt>
              <dd>{show(fact.public_ipv4)}</dd>
            </div>
            <div>
              <dt>Translators on the way out</dt>
              <dd>{fact.nat_layers}</dd>
            </div>
            {fact.tunnels?.map((tunnel) => (
              <div key={tunnel.id}>
                <dt>Mesh address</dt>
                <dd>{tunnel.enabled ? show(tunnel.address) : `${show(tunnel.address)} once joined`}</dd>
              </div>
            ))}
            {node.services?.map((service) => (
              <div key={service.id}>
                <dt>{service.label}</dt>
                <dd>
                  listens on {service.proto.toUpperCase()} {service.port}
                </dd>
              </div>
            ))}
          </>
        )}
        {node.kind === 'router' && (
          <>
            <div>
              <dt>Home side (LAN)</dt>
              <dd>{show(fact.ipv4)}</dd>
            </div>
            <div>
              <dt>Outside (WAN)</dt>
              <dd>{show(fact.wan_ipv4)}</dd>
            </div>
            <div>
              <dt>What the internet sees</dt>
              <dd>{show(fact.public_ipv4)}</dd>
            </div>
            <div>
              <dt>Forgets an idle UDP row after</dt>
              <dd>{node.udp_timeout_s === undefined ? 'unknown' : `${node.udp_timeout_s} s`}</dd>
            </div>
          </>
        )}
        {node.kind === 'isp' && fact.line && (
          <>
            <div>
              <dt>Customers get</dt>
              <dd>
                {fact.line.ipv4 === 'public'
                  ? 'their own public address'
                  : fact.line.ipv4 === 'cgnat'
                    ? 'a shared address behind carrier NAT'
                    : 'unknown'}
              </dd>
            </div>
            {fact.line.ipv4 === 'cgnat' && (
              <div>
                <dt>Shared public address</dt>
                <dd>{show(fact.line.shared_ipv4)}</dd>
              </div>
            )}
            <div>
              <dt>How we know</dt>
              <dd>
                <StatusMark status={fact.line.ipv4 === 'unknown' ? 'unknown' : 'pass'} word={verdictWord(fact.line.ipv4 === 'unknown' ? 'unknown' : 'pass', fact.line.provenance)} />
              </dd>
            </div>
            {profile && (
              <div className="facts__wide">
                <dt>{profile.label}</dt>
                <dd className="facts__note">{profile.note}</dd>
              </div>
            )}
          </>
        )}
      </dl>

      {links.length > 0 && (
        <>
          <h4>Links</h4>
          <ul className="links">
            {links.map((link) => {
              const out = link.a === nodeId ? link.rate_ab_bps : link.rate_ba_bps;
              const back = link.a === nodeId ? link.rate_ba_bps : link.rate_ab_bps;
              return (
                <li key={link.id}>
                  <span>
                    to {other(link.a === nodeId ? link.b : link.a)} by {link.medium === 'backbone' ? 'backbone fibre' : link.medium}
                  </span>
                  <span className="links__numbers">
                    {distance(link.length_m)}, {rate(out)} out, {rate(back)} back
                  </span>
                </li>
              );
            })}
          </ul>
          <p className="result__estimate">Distances and speeds are a teaching model, not a measurement of any real network.</p>
        </>
      )}
    </aside>
  );
}
