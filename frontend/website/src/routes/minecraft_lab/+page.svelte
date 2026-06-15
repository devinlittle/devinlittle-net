<svelte:head>
  <title>DevinLittle.net - Minecraft Server Lab</title>
  <meta name="description" content="hello Sophomore ITS class of 2028! " />
</svelte:head>

<h1>Minecraft Server Lab</h1>

<div>
  <ul>
    <h3>Objectives:</h3>
    <li>Install the Java Runtime Environment (JRE)</li>
    <li>Add Java to the Windows PATH environment variable</li>
    <li>Download and configure a PaperMC server</li>
    <li>Accept the Minecraft EULA</li>
    <li>Configure offline mode</li>
    <li>Start the server and verify it runs correctly</li>
    <li>Obtain your IP address and allow clients to connect</li>
  </ul>
</div>

<div>
  <ul>
    <h3>Required Resources:</h3>
    <li>Java Runtime Enviorment (JRE)</li>
    <li>PaperMC server JAR</li>
    <li>Minecraft client (provided in 7z Archive)</li>
  </ul>
</div>

<h1>Part 1: JRE Setup</h1>

<h2>Step 1: Dowloading Java</h2>
<p>(We’re talking about Java, not Minecraft!)</p>

<a
  target="_blank"
  rel="noopener noreferrer"
  href="https://adoptium.net/temurin/releases?version=21&os=any&arch=any"
  >Java Download Link</a
>

<p>
  Choose the version for your operating system and CPU architecture (most people
  in the IT class will use Windows on x86_64, x64, amd64 [all the same thing
  btw]).
</p>
<p>
  Make sure to download the ZIP file for easy cleanup when finished with lab.
</p>

<img src="minecraft_lab/downloadJRE.gif" alt="" />

<p>
  In this gif, I am running on Windows aarch64, but other than that, the steps
  are the same.
</p>

<h2>Step 2: Extracting and adding to PATH</h2>

<hr style="width:50%;text-align:left;margin-left:0" />

<h3>What is PATH?</h3>
<p>
  the PATH is a list of folders your computer checks when you run a command.
</p>
<ul>
  <li>If Java is on the PATH, typing `java` in a terminal works.</li>
  <li>If it is not, Windows shows:</li>
  <p style="font-weight: bold;">
    “'java' is not recognized as an internal or external command…”
  </p>
</ul>

<p>
  We are adding Java to the PATH so the server knows where the Java Runtime is
  located.
</p>

<hr style="width:50%;text-align:left;margin-left:0" />

<h3>Windows Instructions:</h3>

<ol>
  <li>Create a folder on your desktop, name it "minecraft"</li>
  <li>Extract the Java ZIP to the server folder, for exmaple:</li>
  <pre>C:\Users\Sophomore\Desktop\minecraft\Java\</pre>
  <li>Press Start >> type "Enviorment Varibles" >> open it</li>
  <li>Click "Enviorment Varibles"</li>
  <li>Under "System Varibles", find Path, click Edit > New, and paste</li>
  <pre>C:\Users\Sophomore\Desktop\minecraft\[JAVA FOLDER HERE]\bin</pre>
  <p>
    If you noticed, there is the "bin" folder after the java folder, that is
    there because in the bin folder is all of the java binaries (exetuables).
  </p>
  <li>Click OK on everything</li>
</ol>

<video controls playsinline muted>
  <source src="/minecraft_lab/extractANDenv.mp4" type="video/mp4" />
  Your browser does not support the video tag.
</video>
<p>Here's an exmaple vidoe!</p>

<hr style="width:50%;text-align:left;margin-left:0" />

<h2>Verification!</h2>
<p>Open the Command Prompt and tyep:</p>
<pre>java --version</pre>
<p>continue if you see version info</p>
<p>if it says "not recognized" >> PATH is incorect</p>

<p>Write your result here:</p>
<input type="text" placeholder="Write your answer here" />

<hr style="width:100%;text-align:left;margin-left:0" />

<h1>Part 2: Server Setup</h1>

<ul>
  <p>
    Because Minecraft is so popular, communities have made server software that
    is:
  </p>
  <li>Faster</li>
  <li>Smaller</li>
  <li>Easier to set up</li>
</ul>

<p>
  We'll use PaperMc for this lab. It's bassically the same as a vanilla server
  but with performance improvments.
</p>

<a
  target="_blank"
  rel="noopener noreferrer"
  href="https://papermc.io/downloads/paper">Download PaperMC server here</a
>

<p>
  Put the server JAR file in the server folder you setup in part one and rename
  it to "server.jar"
</p>

<img src="minecraft_lab/download_server.gif" alt="" />

<h2>Step 1: Create a run.bat file</h2>
<p>
  bat files or are just files that when opened, run comamands line by line in
  the command prompt that are defined in the file.
</p>

<ol>
  <li>Create a file named run.txt in the minecraft folder</li>
  <li>Edit with NotePad and add the following to the newly created file:</li>
  <pre>
@echo off
java -Xms6G -Xmx6G -jar server.jar
pause
</pre>
  <li>
    Then in file explorer goto, View >> Show >> and check "File Name Extensions"
  </li>
  <li>now rename the file to "run.bat"</li>
  <li>Click yes</li>
  <li>Double click run.bat</li>
  <p>The server will start</p>
</ol>

<h2>Step 2: Agree to the EULA</h2>

<ol>
  <li>Open the newly created, elua.txt</li>
  <li>Change "false" to "true"</li>
  <p>This is like aggreeing to a ToS</p>
</ol>

<h2>Step 3: Configuring offline mode</h2>

<ol>
  <li>Oepn the server.properties file with NotePad</li>
  <li>Change:</li>
  <pre>online-mode=true</pre>
  <p>to:</p>
  <pre>online-mode=false</pre>
  <p>
    This allows our offline mode clients to connect, without this our clients
    wouldn't be able to authenticate into the server.
  </p>
</ol>

<h2>Step 4: Start the server, fr this time</h2>
<p>Double click the run.bat file again</p>
<p>The server should start!</p>

<video controls playsinline muted>
  <source src="/minecraft_lab/server_setup.mp4" type="video/mp4" />
  Your browser does not support the video tag.
</video>
<p>Exmaple video.</p>

<h3>Checkpoint!</h3>
<p>When the server is done loading. it should print,</p>
<pre>Done (seconds)! For help, type "help"</pre>
<p>Write out the time it took for your server to start:</p>
<input type="text" placeholder="Write your answer here" />

<p>With that your server it setup!</p>
<p>To connect find your ip address and note that down.</p>

<p>
  <a
    target="_blank"
    rel="noopener noreferrer"
    href="https://ranoz.gg/file/cEF8PCDc">minecraft.7z file download</a
  >
  <a>When downloading, right click the download button and open in new tab</a>
  <br />
  <a
    target="_blank"
    rel="noopener noreferrer"
    href="https://home.devinlittle.net:8888/minecraft.7z"
    >slower download if link above doesn't work</a
  >
</p>
<p>This contains the minecraft client (the game)</p>
<p>
  Check with your teacher for premission to continue, they will provide the
  password for the encyrpted file.
</p>

<p>Download and instal 7zip for this archive</p>
<p>right cick the file, click "show more options", 7zip >> open archive</p>
<p>drag the file to C:/</p>

<p>Once the file is downloaded and extracted:</p>
<ol>
  <li>Navigate to the "client_files" folder</li>
  <li>Double Click on "launch.bat"</li>
  <li>minecraft should launch, click multiplyer</li>
  <li>then click direct connect, then type your server IP</li>
</ol>

<h1>Done!</h1>

<p>Complete the following questions:</p>
<ol>
  <li>
    Why does the server generate files and then stop the first time you run it?
  </li>
  <input type="text" placeholder="Write your answer here" />
  <li>What would happen if Java was not added to the PATH?</li>
  <input type="text" placeholder="Write your answer here" />
  <li>What did you find easier or harder than expected?</li>
  <input type="text" placeholder="Write your answer here" />
</ol>
