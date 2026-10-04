-module(ybm_circuit_breaker).
-behaviour(gen_server).

-export([start_link/1, report_success/1, report_failure/1, get_state/1, name/1]).
-export([init/1, handle_call/3, handle_cast/2, handle_info/2,
         terminate/2, code_change/3]).

-record(state, {
    name,
    status = closed,
    failures = 0,
    successes = 0,
    threshold = 5,
    cooldown_ms = 30000,
    success_threshold = 1,
    opened_at = undefined,
    last_transition_at
}).

start_link(Opts) ->
    Name = maps:get(name, Opts),
    gen_server:start_link({local, name(Name)}, ?MODULE, Opts, []).

report_success(Name) ->
    gen_server:cast(name(Name), success).

report_failure(Name) ->
    gen_server:cast(name(Name), failure).

get_state(Name) ->
    gen_server:call(name(Name), get_state).

init(Opts) ->
    {ok, #state{
        name = maps:get(name, Opts),
        threshold = maps:get(threshold, Opts, 5),
        cooldown_ms = maps:get(cooldown_ms, Opts, 30000),
        success_threshold = maps:get(success_threshold, Opts, 1),
        last_transition_at = erlang:system_time(millisecond)
    }}.

handle_call(get_state, _From, State) ->
    {reply, state_map(State), State};
handle_call(_Request, _From, State) ->
    {reply, {error, unsupported}, State}.

handle_cast(success, State = #state{status = closed}) ->
    {noreply, State#state{failures = 0}};
handle_cast(success, State = #state{status = half_open, successes = Successes,
                                    success_threshold = Threshold}) ->
    NewSuccesses = Successes + 1,
    case NewSuccesses >= Threshold of
        true -> transition(closed, State#state{successes = 0, failures = 0});
        false -> {noreply, State#state{successes = NewSuccesses}}
    end;
handle_cast(success, State) ->
    {noreply, State};
handle_cast(failure, State = #state{status = closed, failures = Failures,
                                    threshold = Threshold}) ->
    NewFailures = Failures + 1,
    case NewFailures >= Threshold of
        true -> transition(open, State#state{failures = NewFailures, opened_at = erlang:system_time(millisecond)});
        false -> {noreply, State#state{failures = NewFailures}}
    end;
handle_cast(failure, State = #state{status = half_open}) ->
    transition(open, State#state{opened_at = erlang:system_time(millisecond)});
handle_cast(failure, State) ->
    {noreply, State}.

handle_info(cooldown, State = #state{status = open}) ->
    transition(half_open, State#state{successes = 0});
handle_info(_Info, State) ->
    {noreply, State}.

terminate(_Reason, _State) ->
    ok.

code_change(_OldVsn, State, _Extra) ->
    {ok, State}.

transition(NewStatus, State) ->
    Now = erlang:system_time(millisecond),
    case NewStatus of
        half_open ->
            erlang:send_after(State#state.cooldown_ms, self(), cooldown);
        _ ->
            ok
    end,
    ybm_recovery_coordinator ! {circuit_transition, State#state.name,
                                State#state.status, NewStatus},
    {noreply, State#state{status = NewStatus,
                           last_transition_at = Now,
                           successes = 0}}.

state_map(State) ->
    #{name => State#state.name,
      status => State#state.status,
      failures => State#state.failures,
      successes => State#state.successes,
      last_transition_at => State#state.last_transition_at}.

name(Name) when is_atom(Name) ->
    Name;
name(Name) when is_list(Name) ->
    list_to_atom("ybm_" ++ Name ++ "_circuit");
name(Name) when is_binary(Name) ->
    name(binary_to_list(Name)).
