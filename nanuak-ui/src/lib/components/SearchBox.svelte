<script lang="ts">
  export let onSearch: (query: string) => void;
  let searchQuery = '';
  let typingTimeout: ReturnType<typeof setTimeout> | null = null;

  function handleInput(e: Event) {
    const target = e.target as HTMLInputElement;
    searchQuery = target.value;

    // Clear any existing timeout
    if (typingTimeout) clearTimeout(typingTimeout);

    // Set a new timeout to trigger onSearch after 100ms of no input
    typingTimeout = setTimeout(() => {
      onSearch(searchQuery);
    }, 300);
  }
</script>

<div class="relative">
  <input
    bind:value={searchQuery}
    placeholder="Search your watch history..."
    class="pl-10 h-10 w-full rounded-lg border border-gray-300 shadow-sm focus:ring focus:ring-blue-300 focus:outline-none"
    on:input={handleInput}
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
