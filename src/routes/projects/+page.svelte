<svelte:head>
 <title>DevinLittle.net - Projects</title> 
 <meta name="description" content="Devin's Projects section" />
</svelte:head>

<script>
  import Preview from "$lib/comps/Preview.svelte";
  let { data } = $props();
</script>

<h1>Projects</h1>

<div class="eachLoop">
  {#each data.posts as v}
    <div>
      <br>
        <Preview articleLink={v.path} articleTitle={v.meta.title} articleImage={v.meta.image}/>
      <br>
    </div>
  {/each}
</div>
<!-->Content Here!</!-->

<style>
  .eachLoop {
    display: flex;
    flex-direction: row;
  }

  @media (max-width: 850px) {
  .eachLoop {
    flex-direction: column;
  }
}
</style>
