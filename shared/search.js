import aliases from './search-aliases.json';
// Restricted to common domain words; no remote model or user content transmission.
export function nearWord(a, b) {
  if (!/^[a-z]{4,32}$/.test(a) || !/^[a-z]{4,32}$/.test(b) || Math.abs(a.length-b.length)>1) return false;
  if(a.length===b.length) { const differences=[];for(let i=0;i<a.length;i++)if(a[i]!==b[i])differences.push(i);return differences.length<=1 || (differences.length===2 && differences[1]===differences[0]+1 && a[differences[0]]===b[differences[1]] && a[differences[1]]===b[differences[0]]); }
  const [short,long]=a.length<b.length?[a,b]:[b,a];let i=0;while(i<short.length&&short[i]===long[i])i++;return short.slice(i)===long.slice(i+1);
}
export function searchTerms(query) {
  const needle=String(query ?? '').trim().toLowerCase();if(!needle)return [];
  const exact=aliases.filter(group=>group.includes(needle));
  const groups=exact.length?exact:aliases.filter(group=>group.some(term=>nearWord(needle,term)));
  return [...new Set([needle,...groups.flat()])].slice(0,32);
}
export function matchesSearch(text,query) { const value=String(text??'').toLowerCase(), terms=searchTerms(query);return !terms.length||terms.some(term=>value.includes(term)); }
export function highlightedParts(text,query) {
  text=String(text??'');const lower=text.toLowerCase(), ranges=[];
  for(const term of searchTerms(query))for(let at=lower.indexOf(term);at>=0;at=lower.indexOf(term,at+term.length))ranges.push([at,at+term.length]);
  ranges.sort((a,b)=>a[0]-b[0]);const merged=[];
  for(const range of ranges){const last=merged.at(-1);if(last&&range[0]<=last[1])last[1]=Math.max(last[1],range[1]);else merged.push([...range]);}
  const result=[];let at=0;
  for(const [start,end] of merged){if(start>at)result.push({text:text.slice(at,start),match:false});result.push({text:text.slice(start,end),match:true});at=end;}
  if(at<text.length)result.push({text:text.slice(at),match:false});return result;
}
