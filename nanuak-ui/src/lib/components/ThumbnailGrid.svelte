<script>
  export let videos = [];
  export let onClick = (video) => {};
</script>

<div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
  {#each videos as video}
    <div
      class="group relative overflow-hidden rounded-lg bg-white shadow-sm transition-all hover:shadow-md cursor-pointer"
      on:click={() => onClick(video)}
    >
      <img src={video.thumbnail} alt={video.title} class="h-full w-full object-cover" />
      <div class="absolute bottom-2 right-2 bg-black/80 px-2 py-1 text-xs text-white rounded">
        {Math.floor(video.duration / 60)}:{(video.duration % 60).toString().padStart(2, '0')}
      </div>
      <div class="p-4">
        <h3 class="font-semibold line-clamp-2 mb-2">{video.title}</h3>
        <div class="text-sm text-gray-600">
          {video.views} views
        </div>
      </div>
    </div>
  {/each}
</div>
