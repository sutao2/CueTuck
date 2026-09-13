"""Isolated 50k-row stdio benchmark; optional old executable as first argument."""
import sqlite3,tempfile,subprocess,os,json,time,statistics,sys
from pathlib import Path
with tempfile.TemporaryDirectory() as d:
 c=sqlite3.connect(str(Path(d)/'promptark.sqlite'))
 c.execute('CREATE TABLE prompts(id TEXT PRIMARY KEY,title TEXT,summary TEXT,content TEXT,category_id TEXT,model TEXT,deleted_at TEXT)')
 c.executemany('INSERT INTO prompts VALUES(?,?,?,?,?,?,NULL)',((f'p{i:06}', f'样本 {i:06}'+(' 写作' if i%503==0 else ''), '模拟摘要',('请根据用户输入整理结构和细节，输出准确简洁的内容。' * 15)+f' sample{i:06} '+('自然光 摄影 portrait' if i%701==0 else '说明文档'), 'text','gpt') for i in range(50000)))
 c.commit(); c.close()
 binaries = sys.argv[1:] + [str(Path(__file__).resolve().parents[1]/'target/release/promptark-mcp')]
 for binary in binaries:
  p=subprocess.Popen([binary],env={**os.environ,'PROMPTARK_LIBRARY_DIR':d,'PROMPTARK_MCP_SQUARE':'0'},stdin=subprocess.PIPE,stdout=subprocess.PIPE,text=True)
  def search(q):
   t=time.perf_counter();p.stdin.write(json.dumps({'jsonrpc':'2.0','id':1,'method':'tools/call','params':{'name':'search_prompts','arguments':{'query':q}}})+'\n');p.stdin.flush();r=json.loads(p.stdout.readline());ms=(time.perf_counter()-t)*1000
   if r['result'].get('isError'):raise RuntimeError(r)
   return ms,len(json.loads(r['result']['content'][0]['text']))
  print(Path(binary).name, 'cold_ms', round(search('写作')[0],2),flush=True)
  for q in ['写作','自然光','不存在XYZ','写作 portrait','']:
   times=[search(q)[0] for _ in range(11)]
   print(repr(q),'median_ms',round(statistics.median(times),2),'max_ms',round(max(times),2),flush=True)
  p.stdin.close();p.wait()
