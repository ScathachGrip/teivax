fn main() -> Result<(), Box<dyn std::error::Error>> {
    let html = generate_html();
    let size = html.len();
    std::fs::create_dir_all("playground")?;
    std::fs::write("playground/index.html", &html)?;
    println!(
        "wrote playground/index.html  ({:.1} KiB)",
        size as f64 / 1024.0
    );
    Ok(())
}

fn generate_html() -> String {
    let mut h = String::new();

    // --- HTML head ---
    h.push_str(r#"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="UTF-8">
<meta name="viewport" content="width=device-width,initial-scale=1.0">
<title>teivax — API Playground</title>
<style>
*{margin:0;padding:0;box-sizing:border-box}
body{font-family:-apple-system,BlinkMacSystemFont,'Segoe UI',Roboto,Oxygen,Ubuntu,Cantarell,sans-serif;background:#0d1117;color:#c9d1d9;min-height:100vh}
.container{max-width:720px;margin:0 auto;padding:24px 16px}
h1{font-size:24px;font-weight:600;color:#f0f6fc;margin-bottom:4px}
.sub{color:#8b949e;font-size:14px;margin-bottom:24px}
.toolbar{display:flex;gap:8px;margin-bottom:24px;flex-wrap:wrap}
.toolbar input{flex:1;min-width:200px;padding:10px 14px;border:1px solid #30363d;border-radius:8px;font-size:14px;background:#161b22;color:#c9d1d9;outline:none;transition:border-color .2s}
.toolbar input:focus{border-color:#58a6ff}
.toolbar button{padding:10px 20px;background:#238636;color:#fff;border:none;border-radius:8px;font-size:14px;font-weight:500;cursor:pointer;transition:background .2s}
.toolbar button:hover{background:#2ea043}
.loading{text-align:center;color:#8b949e;padding:40px;font-size:14px}
.card{display:flex;align-items:center;gap:14px;padding:14px 16px;border:1px solid #30363d;border-radius:8px;margin-bottom:8px;transition:border-color .2s}
.card:hover{border-color:#484f58}
.card.ok{border-left:3px solid #238636}
.card.err{border-left:3px solid #da3633}
.status{font-size:18px;font-weight:700;width:24px;text-align:center}
.card.ok .status{color:#3fb950}
.card.err .status{color:#f85149}
.info{display:flex;flex-direction:column;gap:2px}
.title{color:#58a6ff;text-decoration:none;font-weight:500;font-size:15px}
.title:hover{text-decoration:underline}
.meta{color:#8b949e;font-size:12px}
.info-card{border-color:#21262d;border-left:3px solid #58a6ff}
.info-card .info a{color:#58a6ff;text-decoration:none}
.info-card .info a:hover{text-decoration:underline}
@media(max-width:480px){.toolbar{flex-direction:column}.toolbar input{min-width:auto}}
</style>
</head>
<body>
<div class="container">
<h1>teivax</h1>
<div class="sub">Anime character tag registries</div>
<div class="toolbar">
<input type="text" id="base-url" value="http://localhost:3000">
<button onclick="apply()">Apply</button>
</div>
<div id="output"></div>
</div>
<script>
"#);

    // --- JS ---
    h.push_str(r#"
const entries = [
  {id:"nikke",title:"Nikke",count:106},
  {id:"arknights",title:"Arknights",count:77},
  {id:"bluearchive",title:"Blue Archive",count:120},
  {id:"fgo",title:"Fate/Grand Order",count:85},
  {id:"genshin",title:"Genshin Impact",count:142},
  {id:"genshin_danbooru",title:"Genshin Impact (Danbooru)",count:98},
  {id:"azurlane",title:"Azur Lane",count:65},
  {id:"girls_frontline",title:"Girls' Frontline",count:72},
  {id:"global_anime_girls",title:"Global Anime Girls",count:200},
  {id:"hentai_yandere",title:"Hentai (Yandere)",count:54},
  {id:"danbooru_sex",title:"Danbooru Sex",count:48},
  {id:"gif_sex",title:"GIF Sex",count:30},
  {id:"bleach",title:"Bleach",count:40},
  {id:"blocklists",title:"Blocklists",count:25},
];

function apply(){
  const base=document.getElementById('base-url').value.replace(/\/+$/,'');
  document.getElementById('output').innerHTML='<div class="loading">Loading...</div>';
  renderEntries(base);
}
async function renderEntries(base){
  let html='';
  for(const e of entries){
    const ok = await testEndpoint(base,e.id);
    html+=`<div class="card ${ok?'ok':'err'}"><div class="status">${ok?'✓':'✗'}</div><div class="info"><a href="${base}/${e.id}" target="_blank" class="title">${e.title}</a><span class="meta">/${e.id} · ${e.count} tags</span></div></div>`;
  }
  html+=`<div class="card info-card"><div class="info">Endpoints: <a href="${base}/health" target="_blank">/health</a> · <a href="${base}/loadavg" target="_blank">/loadavg</a> · <a href="${base}/metrics" target="_blank">/metrics</a></div></div>`;
  document.getElementById('output').innerHTML=html;
}
async function testEndpoint(base,id){
  try{
    const r=await fetch(base+'/'+id,{signal:AbortSignal.timeout(3000)});
    if(r.ok){const d=await r.json();return Array.isArray(d)&&d.length>0}
    return false;
  }catch{return false}
}
apply();
"#);

    // --- tail ---
    h.push_str("</script>\n</body>\n</html>\n");
    h
}
