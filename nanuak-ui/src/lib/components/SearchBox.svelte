<script>
  export let onSearch = (query) => {};
  let searchQuery = '';

  const handleSearch = () => {
    onSearch(searchQuery);
  };
</script>

<div class="relative">
  <input
    bind:value={searchQuery}
    placeholder="Search your watch history..."
    class="pl-10 h-10 w-full rounded-lg border border-gray-300 shadow-sm focus:ring focus:ring-blue-300 focus:outline-none"
    on:keypress={(e) => e.key === 'Enter' && handleSearch()}
  />
  <svg
    class="absolute left-3 top-2.5 h-5 w-5 text-gray-400"
    xmlns="http://www.w3.org/2000/svg"
    fill="none"
    viewBox="0 0 24 24"
    stroke="currentColor"
  >
    <path
      stroke-linecap="round"
      stroke-linejoin="round"
      stroke-width="2"
      d="M10 19l-2-2m0 0a7 7 0 1114 0 7 7 0 01-14 0z"
    />
  </svg>
</div>
