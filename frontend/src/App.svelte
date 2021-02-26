<style lang="scss" global>
  @import "./global.scss";
</style>

<script>
  import session from './session';
  import SessionOverlay from './SessionOverlay.svelte';
  import Chat from './Chat.svelte';
  import { onDestroy } from 'svelte';
  import StartButton from './StartButton.svelte';

  let sessionState = session.getState();
  $: status = sessionState.status;
  $: jwt = sessionState.jwt || localStorage.getItem('token');

  const unsubscribe = session.getStore().subscribe((state) => {
    sessionState = state;

    if (state.jwt) {
      localStorage.setItem('token', state.jwt);
    }
  });

  function logout() {
    localStorage.removeItem('token');
    session.reset();
  }

  onDestroy(unsubscribe);
</script>

{#if jwt}
  <Chat {jwt} {logout} />
{:else if status !== null}
  <SessionOverlay {sessionState} cancel={() => session.reset()} />
{:else}
  <StartButton loading={status} start={() => session.start()} />
{/if}
