<script>
  import Message from './Message.svelte';
  import { onMount } from "svelte";
  export let jwt;
  export let logout;

  const formatDate = new Intl.DateTimeFormat('en', {
    dateStyle: 'medium',
    timeStyle: 'short',
    hour12: false,
  }).format;

  let messages = [];

  let input;

  onMount(() => input.focus());

  let newMessage = '';

  const host = window.location.host;
  const socket = new WebSocket(`wss://${host}/chat`);
  socket.addEventListener('open', () => {
    socket.send(jwt)
  });

  socket.addEventListener('message', (event) => {
    const message = JSON.parse(event.data);

    if (message.error) {
      logout();
    } else {
      messages = [{
        ...message,
        time: formatDate(new Date(message.time * 1000)),
      }, ...messages];
    }
  });

  function sendMessage(event) {
    event.preventDefault();
    if (socket.OPEN && newMessage) {
      socket.send(newMessage)
      newMessage = '';
    }
  }
</script>

<style lang="scss">
  main {
    display: flex;
    flex-direction: column;
    height: 100%;
    margin: 0 auto;
    padding: 0;
  }

  header {
    background-color: #eee;
    padding: 1rem;
    margin: 0;
    display: flex;
    align-items: center;
    justify-content: space-between;
    align-content: center;

    button {
      cursor: pointer;
    }

    h2 {
      margin: 0;
      font-weight: bold;
    }
  }

  ul {
    display: flex;
    flex-direction: column-reverse;
    padding: 0;
    margin: 0;
    overflow-y: scroll;
    background-color: #f6f6f6;
    flex: 1;
  }

  form {
    padding: 1rem;
    background-color: #eee;

    label {
      display: none;
    }

    input {
      width: 100%;
    }
  }
</style>

<main>
  <header>
    <h2>IRMA Chat</h2>
    <button on:click={logout}>
      Logout
    </button>
  </header>
  <ul>
    {#each messages as message}
      <Message message={message} />
    {/each}
  </ul>
  <form on:submit={sendMessage}>
    <label for="message" id="message-label">
      Message
    </label>
    <input
      type="text"
      name="message"
      id="message"
      bind:this={input}
      bind:value={newMessage}
      placeholder="Enter your message"
      aria-labelledby="message-label"
    />
  </form>
</main>
