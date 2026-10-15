"""Read-only PDF audit. Writes application corrections; never edits supplied data.

Dot counts and colored rank icons are vector paths, not extractable text. Ambiguous
name matches / icons remain unresolved rather than borrowing a different form.
Requires pdfplumber; run with the workspace Python runtime.
"""
import json, re, hashlib
from pathlib import Path
import pdfplumber

APP = Path(__file__).resolve().parents[1]
DATA = APP.parent / "Pokerole-Data"
PDF = DATA / "POKEROLE COREBOOK 3.0 POKEDEX MASTER.pdf"
ATTRS = ["Strength", "Dexterity", "Vitality", "Special", "Insight"]
RANK_COLORS = {(1., 1., 1.): "Starter", (.904, .201, .177): "Rookie",
               (.402, .745, .887): "Standard", (.933, .804, .221): "Advanced"}

def norm(s):
    for a,b in [("alolan", "alola"), ("galarian", "galar"), ("hisuian", "hisui"),
                ("paldean", "paldea"), ("form", ""), ("forme", "")]:
        s = s.lower().replace(a,b)
    return re.sub(r"[^a-z0-9]", "", s)

entries = [json.loads(f.read_text(encoding="utf-8")) for f in (DATA / "v3.0/Pokedex").glob("*.json")]
by_num = {}
for e in entries: by_num.setdefault(int(e["Number"]), []).append(e)
corrections, unresolved, differences = {}, [], []

def color(c):
    v = c.get("non_stroking_color")
    return tuple(round(x,3) for x in v) if isinstance(v, (tuple,list)) else v

def dark(c):
    v=color(c)
    return isinstance(v,tuple) and max(v)<.35 and max(v)-min(v)<.03

with pdfplumber.open(PDF) as doc:
    for page in doc.pages:
        words = page.extract_words()
        headers = sorted([w for w in words if re.fullmatch(r"#\d{3,4}", w["text"]) and w['height']>12], key=lambda w:w["top"])
        if not headers or not any(w["text"].lower() == "strength" for w in words):
            page.close(); continue
        curves = page.curves
        for i,h in enumerate(headers):
            top = h["top"]-2
            bottom = headers[i+1]["top"]-3 if i+1<len(headers) else page.height-15
            ws = [w for w in words if top<=w["top"]<bottom]
            title = " ".join(w["text"] for w in ws if abs(w["top"]-h["top"])<6 and w["x0"]>=h["x1"]-1 and w["x0"]<h["x1"]+155 and w["text"] not in ["Type:","TYPE:"])
            title = re.split(r"\b(?:Type:|Rank|Normal|Electric|Water|Grass|Fire|Rock|Ground|Bug|Steel|Ghost|Fairy|Psychic|Flying|Poison|Dragon|Dark|Ice|Fighting)\b",title)[0].strip()
            candidates = by_num.get(int(h["text"][1:]), [])
            exact = [e for e in candidates if norm(e["Name"])==norm(title)]
            if len(exact)!=1:
                unresolved.append({"page":page.page_number,"name":title,"reason":"No unique exact form/name match"}); continue
            e = exact[0]; key=f'{e["DexID"]}:{e["_id"]}'
            values = {}; ok=True
            for a in ATTRS:
                labels = [w for w in ws if w["text"].lower()==a.lower()]
                if len(labels)!=1: ok=False; break
                w=labels[0]
                dots=[c for c in curves if c["fill"] and 5.8<c["width"]<7.2 and 5.8<c["height"]<7.2 and w["x1"]<c["x0"]<w["x0"]+140 and abs((c["top"]+c["bottom"])/2-(w["top"]+w["bottom"])/2)<5]
                # The PDF paints updated values over earlier dots. Last path wins,
                # exactly as the renderer does; duplicate dots must not add points.
                dots=list({(round(c['x0'],1),round(c['top'],1)):c for c in dots}.values())
                if not dots or any(color(c)!=(1.,1.,1.) and not dark(c) for c in dots): ok=False; break
                values[a]=sum(dark(c) for c in dots)
                values['Max'+a]=len(dots)
            hp=[w for w in ws if w['text'].isdigit() and 20<w['height']<30]
            if len(hp)==1: values['BaseHP']=int(hp[0]['text'])
            abilities=[]
            labels=[w for w in ws if w['text'].lower()=='abilities:']
            if len(labels)==1:
                label=labels[0]
                aw=sorted([w for w in ws if label['top']+5<w['top']<label['top']+30 and label['x0']-2<w['x0']<label['x0']+143 and w['height']>10],key=lambda w:(round(w['top']),w['x0']))
                raw=' '.join(w['text'] for w in aw)
                abilities=[a.strip() for a in raw.split('&') if a.strip()]
                if 0<len(abilities)<=2: values.update({'Ability1':abilities[0], 'Ability2':abilities[1] if len(abilities)>1 else ''})
            # Species stage is explicitly printed, including standalone/final species.
            stage_words=[w for w in ws if w['text'] in ['First.','Second.','Final.']]
            if len(stage_words)==1: stage={'First.':1,'Second.':2,'Final.':3}[stage_words[0]['text']]
            else: stage=None
            # Match each move name against the text at its colored rank icon.
            moves=[]; unknown=[]
            for m in e['Moves']:
                parts=m['Name'].split()
                first=[w for w in ws if w['text'].lower()==parts[0].lower() and w['height']<10]
                matches=[]
                for w in first:
                    line=sorted([v for v in ws if abs(v['top']-w['top'])<1 and w['x0']-.1<=v['x0']<w['x0']+145],key=lambda v:v['x0'])
                    if not " ".join(v['text'] for v in line).startswith(m['Name']): continue
                    icons=[c for c in curves if c['fill'] and 7<c['width']<9.3 and 3.4<c['height']<4.3 and w['x0']-65<c['x0']<w['x0']-42 and abs(c['top']-w['top'])<6]
                    if len(icons)<2: continue
                    lo=min(c['top'] for c in icons)
                    uppers=[c for c in icons if c['top']<lo+2]
                    lowers=[c for c in icons if c['top']>=lo+2]
                    if not uppers or not lowers: continue
                    upper,lower=color(uppers[-1]),color(lowers[-1])
                    # White lower half: Starter/Rookie/Standard/Advanced. Both colored: Expert/Ace.
                    if lower==(1.,1.,1.):
                        if upper==(1.,1.,1.): r='Starter'
                        elif isinstance(upper,tuple) and upper[0]>.75 and upper[1]<.4: r='Rookie'
                        elif isinstance(upper,tuple) and upper[2]>.6 and upper[0]<.7: r='Standard'
                        elif isinstance(upper,tuple) and upper[0]>.7 and upper[1]>.5: r='Advanced'
                        else: continue
                    elif isinstance(upper,tuple) and upper[1]>.4 and upper[0]<.6: r='Expert'
                    elif isinstance(upper,tuple) and upper[0]>.7 and upper[1]<.4: r='Ace'
                    else: continue
                    matches.append(r)
                if len(matches)==1: moves.append({'Name':m['Name'],'Learned':matches[0]})
                else: unknown.append(m['Name']); moves.append(m)
            if not ok: unresolved.append({'page':page.page_number,'name':e['Name'],'reason':'Attribute dot recognition uncertain'}); continue
            changed={k:{'json':e.get(k),'pdf':v} for k,v in values.items() if e.get(k)!=v}
            changed_moves=[{'move':m['Name'],'json':old['Learned'],'pdf':m['Learned']} for m,old in zip(moves,e['Moves']) if m['Learned']!=old['Learned']]
            if changed or changed_moves: differences.append({'species':e['Name'],'page':page.page_number,'attributes':changed,'moves':changed_moves})
            lines=page.extract_text().splitlines()
            evolution_lines=[line.strip() for line in lines if re.search(r'evol(?:ut|v)',line,re.I)]
            corrections[key]={'fields':{**values,'Moves':moves},'stage':stage,'pdf_page':page.page_number,'unresolved_moves':unknown,'evolution_source_lines':evolution_lines,'audit':'verified-vector-fields' if not unknown and stage and 'BaseHP' in values and 'Ability1' in values else 'partial-vector-audit'}
        if page.page_number%25==0: print(f'Audited {page.page_number} pages: {len(corrections)} matched species',flush=True)
        page.close()
report={'version':1,'source':PDF.name,'source_sha256':hashlib.sha256(PDF.read_bytes()).hexdigest(),'method':'Exact form/name matching; PDF vector dot counts and rank icon colors; ambiguous fields require adjudication. Evolution narrative/ability exceptions remain manual.','species':corrections,'differences':differences,'unmatched_pdf_entries':unresolved,'unmatched_catalog_species':[e['Name'] for e in entries if f'{e["DexID"]}:{e["_id"]}' not in corrections]}
out=APP/'rules/catalog-corrections.json'; out.parent.mkdir(exist_ok=True); out.write_text(json.dumps(report,ensure_ascii=False,indent=2),encoding='utf-8')
print(f'{len(corrections)}/{len(entries)} species audited; {len(differences)} corrections; {len(unresolved)} unresolved PDF entries; {out}')
