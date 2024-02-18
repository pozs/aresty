<ol>
{{#for i in ints}}
    <li class="{{#if i % 6 == 0}}fizzbuzz{{#else if i % 2 == 0}}fizz{{#else if i % 3 == 0}}buzz{{#else}}none{{/if}}">{{i}}</li>
{{/for}}
</ol>
<ul>
{{#for o in opts}}
    <li>
        {{#match o}}
        {{=None}}Nothing at all
        {{=Some(s)}}It is a "{{s}}"
        {{/match}}
    </li>
{{/for}}
</ul>
