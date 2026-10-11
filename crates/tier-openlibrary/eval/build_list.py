"""Pin the 50-book list: one free-text search per book (q=title author,
limit 1), expected LC = the most frequent class number among the top
work's lcc call numbers. Paced at 1 request per 1.1 s."""
import json, sys, time, urllib.parse, urllib.request, collections, re

UA = "shelfmark/0.1.0 (+https://gitlab.com/davidawad/shelfmark)"
BOOKS = [
 ("Pride and Prejudice","Jane Austen"),("Moby Dick","Herman Melville"),
 ("Nineteen Eighty-Four","George Orwell"),("The Great Gatsby","F. Scott Fitzgerald"),
 ("To Kill a Mockingbird","Harper Lee"),("The Catcher in the Rye","J. D. Salinger"),
 ("Brave New World","Aldous Huxley"),("Frankenstein","Mary Shelley"),
 ("Dracula","Bram Stoker"),("War and Peace","Leo Tolstoy"),
 ("Crime and Punishment","Fyodor Dostoyevsky"),("On the Origin of Species","Charles Darwin"),
 ("A Brief History of Time","Stephen Hawking"),("The Selfish Gene","Richard Dawkins"),
 ("Silent Spring","Rachel Carson"),("The Structure of Scientific Revolutions","Thomas S. Kuhn"),
 ("Cosmos","Carl Sagan"),("The Double Helix","James D. Watson"),
 ("Gödel, Escher, Bach","Douglas R. Hofstadter"),("The Art of Computer Programming","Donald E. Knuth"),
 ("Structure and Interpretation of Computer Programs","Harold Abelson"),("The C Programming Language","Brian W. Kernighan"),
 ("Design Patterns","Erich Gamma"),("The Mythical Man-Month","Frederick P. Brooks"),
 ("Introduction to Algorithms","Thomas H. Cormen"),("The Wealth of Nations","Adam Smith"),
 ("The General Theory of Employment, Interest and Money","John Maynard Keynes"),("Das Kapital","Karl Marx"),
 ("The Road to Serfdom","F. A. Hayek"),("Thinking, Fast and Slow","Daniel Kahneman"),
 ("Guns, Germs, and Steel","Jared Diamond"),("The Rise and Fall of the Third Reich","William L. Shirer"),
 ("The Guns of August","Barbara W. Tuchman"),("The Republic","Plato"),
 ("Leviathan","Thomas Hobbes"),("Critique of Pure Reason","Immanuel Kant"),
 ("Thus Spoke Zarathustra","Friedrich Nietzsche"),("The Prince","Niccolò Machiavelli"),
 ("The Second Sex","Simone de Beauvoir"),("The Interpretation of Dreams","Sigmund Freud"),
 ("The Feminine Mystique","Betty Friedan"),("Democracy in America","Alexis de Tocqueville"),
 ("The Souls of Black Folk","W. E. B. Du Bois"),("Walden","Henry David Thoreau"),
 ("Leaves of Grass","Walt Whitman"),("Hamlet","William Shakespeare"),
 ("The Odyssey","Homer"),("Don Quixote","Miguel de Cervantes"),
 ("Joy of Cooking","Irma S. Rombauer"),("The Elements of Style","William Strunk"),
 ("A People's History of the United States","Howard Zinn"),("The Hobbit","J. R. R. Tolkien"),
 ("Silent Spring","Rachel Carson"),("The Communist Manifesto","Karl Marx"),
 ("Freakonomics","Steven D. Levitt"),
]
SORT = re.compile(r"^([A-Z]{1,3})-*0*(\d+)(?:\.(\d+))?")
def class_number(lcc):
    m = SORT.match(lcc.strip())
    if not m: return None
    letters, whole, dec = m.groups()
    dec = (dec or "").rstrip("0")
    return f"{letters}{whole}" + (f".{dec}" if dec else "")

SUBCLASSES = {c["code"] for c in json.load(open(sys.argv[1]))["classes"] if c["code"].isalpha()}
seen, out = set(), []
for title, author in BOOKS:
    if (title, author) in seen: continue
    seen.add((title, author))
    q = urllib.parse.urlencode({"q": f"{title} {author}", "fields": "key,title,author_name,lcc", "limit": 1})
    req = urllib.request.Request(f"https://openlibrary.org/search.json?{q}", headers={"User-Agent": UA})
    try:
        doc = json.load(urllib.request.urlopen(req, timeout=20))["docs"]
    except Exception as e:
        print("skip", title, e, file=sys.stderr); time.sleep(1.1); continue
    time.sleep(1.1)
    if not doc: print("skip none", title, file=sys.stderr); continue
    d = doc[0]
    nums = [c for c in (class_number(x) for x in d.get("lcc", [])) if c and re.match(r"[A-Z]+", c).group() in SUBCLASSES]
    if not nums: print("skip nolcc", title, d["key"], file=sys.stderr); continue
    top, n = collections.Counter(nums).most_common(1)[0]
    out.append({"title": title, "authors": [author], "expected_lc": top,
                "expected_share": round(n/len(nums), 3), "record": d["key"], "record_title": d.get("title")})
    print(len(out), title, top, d["key"], file=sys.stderr)
    if len(out) == 50: break
json.dump({"retrieved": time.strftime("%Y-%m-%d"), "source": "https://openlibrary.org/search.json (q=title author, limit=1)",
           "license": "Open Library data, CC0 (https://openlibrary.org/developers/licensing)",
           "books": out}, sys.stdout, indent=1, ensure_ascii=False)
