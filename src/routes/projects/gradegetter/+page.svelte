<h1>My GradeGetter Project</h1>

<article>
  <p class="postDate">Wrote on 2025-09-21</p>
  <a href="/projects">Back to Projects</a>
</article>
<p>
  I got tired of using Schoology's outdated ui, so I built my own grade viewer
  in Rust. This post is about how I reverse engineered it, kept it cross
  platform, refused to use AI for shortcuts, and crafted a backend, frontend,
  and desktop app from scratch.
</p>

<p>
  I started this project because the high school I go to uses a platform called
  "Schoology" which is just ASS. Schoology has a very jQuery era look to it and
  looks really like a 2010s java app that somone forgot to redesign for the
  modern era. Schoology has a very frustrating way to show you your grades,
  having to click small text to open cascading boxes, then having scroll for a
  while is NOT good UX. AT. ALL. It was so bad I dedicated a lot of my summer,
  and sepember to studying systems programing, rust, linux, and probably more
  just to get this project aflout. I came in knowing a lot having like 5 years
  in the IT/prgrammming scene, I mean you can see that in the front page of my
  site advertising my "15+ projects" they were never this big. I also got into
  monorepos and that would signifcantly decreese my "15+" if they were
  monorepos, more like "3 1/2" BUT HEY, they were good.
</p>

<p>
  Now most of the time I end up telling people "I started this project like last
  month.". Well this is tenchically true acording to the git commit log...I
  really started it towards the end of last school year (April-May-June/2025 I
  just can't remeber when), I did a lot of the hard work back then:
</p>

<ul>
  <li>The Idea</li>
  <li>The Reverse Engineering</li>
  <li>A simple rust app that returned my grades in html!</li>
</ul>
<br />

<p>
  Sounds great right! At the time, I wasn’t impressed with it myself. How would
  I access this at school? How could I make this more polished? How can I make
  my grades live update? Then it clicked. This would be a great project to do.
  My rules for the project I then simply laid out.
</p>

<ul>
  <li>1. No AI (AT ALL)</li>
</ul>

<p>
  Recently I've seen the trend of "vibe coding" where people just use AI to do
  everything for them. I didn't want to do that. How am I gonna improve as a
  developer with AI just giving me the answers. Everything from the deployment,
  to the final product was made without AI. All though I would like to mention
  that AI did help me learn a lot of rust in the process by quizing me and
  telling me topics to further reasearch. But does that really count? I dont
  think so.
</p>

<ul>
  <li>ANNND 2. Make it Cross Platform</li>
</ul>

<p>
  Simple right! NO. Ever since I've started getting into computers (at the ripe
  age of about...6-7? AHAHAHA|HAH LOLOOLLL but fr i was like 6-7 years old) I've
  been interested into making my...game? App? Whatever it is to work on every
  platform. So with that, I wanted the same for this project, a website, native
  app, and mobile app.
</p>
<p>
  The idea is that it would be intutive enough for users to be hooked and stay
  on the platform due to its cross device nature and just...its a good product.
  HOPEFULLY. I have no users yet because this is still version 0.0.1 as of
  writing BUT, the app works, and just needs polish (as in css), a mobile app
  (I've not tried for this due to the $100/year ask from apple and the lack of
  android users in the US). But, the mobile app will have a wiget of your
  grades, and honestly what ever else I might add on in the future. The web
  version can just be a tab you can pin on your chromebook in school and just
  Ctrl-Tab over to whenever you want to see your grades.
</p>

<h2 style="text-align: center;">
  Consistantly work on this damn project Devin.
</h2>

<p>
  Was defiantly something I've told my self too many times this week. I was
  treating this project as a crush. Thinking about it weather I was at a friends
  house, scroling tiktok, listening to music, or in the damn shower.
</p>

<p>
  I was once in litteral hell, Migraine AND A Ear Infection, so basically my
  head was in terror, and still I found a way to think of ways to impliment
  multi-threading and Read-Write locks into this proeject 😭
</p>

<p>Heres an acient artifact of what the project looked elier on.</p>

<div class="video-container">
  <video controls playsinline muted>
    <source src="/gradegetter/oldversion.mp4" type="video/mp4" />
    Your browser does not support the video tag.
  </video>
  <p class="video-caption">
    Snap sent to my friend · August 13th, 2025 · 1:16 PM
  </p>
  <p class="video-caption">
    Sorry for the "bad quality" because I had to compress it.
  </p>
</div>

<p>
  Pretty impresive right? At the time it still was. The underlying logic there
  was, open pupeteer → extract session token (really a cookie) → send a prehtera
  of requests until → parse grades into JSON → output to stdout
</p>

<p>
  And honestly, that is kinda still how the logic will work for 100 users. The
  core problem here was for the token part, it was resource intesive due to
  having to open a puppeteer instance. Puppeteer attaches to the chrome browser
  to automate whatever you want, and thats how I uesd it but chrome itself is
  really resource intensive. So those requets to create a user end up taking
  about ~10s just for puppeteer to do its thing.
</p>

<p>
  Since I'm in the IT shop at my vocational school I asked the people there at
  the first day at school. "Look what I worked on this summer", as I used this
  project as some sort of showcase of all the knowledge I learned this summer by
  studying HARD. I also wanted to show them something I found kinda cool back
  then. So I asked them the question "Would you guys WANT to use this to show
  your grades in something like a google doc or on your phone more intutively?".
  They all unanimusly said yes with the same reasons I said before, schoology
  having bad UX, and an esspeically bad mobile app.
</p>

<p>So I started starting testing.</p>

<div class="snap-container">
  <img src="/gradegetter/testing-phase.png" alt="" />
  <p class="snap-caption">
    Snap sent to my friend · August 16th, 2025 · 1:16 PM
  </p>
</div>

<p>
  As you can see this is a screenshot of snap I sent like a while ago. And thats
  when I made a little testing script to see how long a token lasts for.
  Information I would use later on.
</p>

<p>
  My idea, which sounds super insecure and just a databreach away from leaking
  all my users infomation, SOUNDS CRAZY BUT YOU GOTTA HEAR ME OUT. Why don't I
  store the users information in my own database, encrypt it, decrypt it on the
  server with secrets I CAN'T EVEN SEE, and store the token, then constatly
  update that token so its always a live token AND BOOM, I can just access their
  grades whenever I want and give those quick speedy updates, that can only
  kinda be revialed by spamming the refresh button but still, if they did that
  on the schoology website, they gotta oepn up all those boxes they worked so
  hard to open. Anyways, my idea should, key word should, work. Again, I had to
  do reverse engineering in the first place because schoology doesn't give out
  their API keys, well at least to my school and I would have to ask my IT
  director for more them, but they would probably say no, and if I already asked
  and they said no, I am scrared to know what type of trouble I would be in when
  they figure out that I decided I didn't need their API key.
</p>

<p>
  Just to not scare potential users away, I created my own library to
  AES-256-GCM encrypt and decrypt strings, If anyone ever got into my database,
  Its not game over for really any of us, the most they have would be your
  username, and the time you created your account. Litteraly everything else is
  encrypted and when decrpted is safely being used and then dropped. I didn't
  put in so much effort studying just to insecurly handle user data.
</p>

<p>
  Honestly This project is just too big to cover in 1 day today, atleast for me
  because I have things to do. So I need to get the minimum of 1 git commit per
  day. I don't care about the typos and it adds character to me. Il Update u
  later..byeee byee
</p>

<style>
  .snap-container {
    text-align: center;
    margin: 40px 0;
  }

  .snap-container img {
    max-height: 600px;
    max-width: 90vw;
    width: auto;
    height: auto;
    border-radius: 12px;
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
  }

  .snap-caption {
    font-size: 0.85rem;
    color: #666;
    margin-top: 8px;
    font-style: italic;
  }

  .video-container {
    text-align: center;
    margin: 40px 0;
  }

  .video-container video {
    max-height: 600px;
    max-width: 90vw;
    border-radius: 12px;
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
  }

  .video-caption {
    font-size: 0.85rem;
    color: #666;
    margin-top: 8px;
    font-style: italic;
  }
  .postDate {
    font-size: 85%;
    color: darkgray;
  }
</style>
