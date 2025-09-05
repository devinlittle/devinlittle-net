<!-->$lib/comps/Preview.svelte</!-->

<script>
  /**
   * @typedef {Object} Props
   * @property {string} [articleLink]
   * @property {string} [articleTitle]
   * @property {string} [articleImage]
   */

  /** @type {Props} */
  let { articleLink = "N/A", articleTitle = "N/A", articleImage = "n/a" } = $props();
</script>

<article class="articleClass">
  <div class="articleTitle">
    <h1>
      <a href="{articleLink}">
        {articleTitle}
      </a>
    </h1>
  </div>
  <br/>
  <br/>
  <hr style="color: blue;"/>
  <img src="proimgs/{articleImage}" alt="Article Logo"/>
</article>

<style>
  .articleClass {
    border: 3.2px solid var(--color-bg-1);
    border-radius: 5px;
    width: 65%;
    height: 95%;
  }
  .articleTitle {
    width: 100%;
    height: 50px;
  }

  img {
    width: 50%;
    display: grid;
    margin-left: auto;
    margin-right: auto;
    width: 50%;
  }

</style>
