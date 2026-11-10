// AGPL network clause: the running site must link to its corresponding source.
// Both values are set by the deploy workflow.
const url = import.meta.env.VITE_SOURCE_URL;
const commit = import.meta.env.VITE_COMMIT;

export const sourceLink = url
  ? {
      href: commit ? `${url}/tree/${commit}` : url,
      label: commit ? `Source (${commit.slice(0, 7)})` : 'Source',
    }
  : null;
