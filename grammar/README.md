# Grammar Classifier

```shell
cargo run < example.txt
```

Example input:
```
G[N]
N, D
N::=ND|D
D::=0|1|2|3|4|5|6|7|8|9
```

Example output:
```
S: NonTerminal('N')
v_n: {'N', 'D'}
p: Production { head: [NonTerminal('N')], body: [[NonTerminal('N'), NonTerminal('D')], [NonTerminal('D')]] }
p: Production { head: [NonTerminal('D')], body: [[Terminal('0')], [Terminal('1')], [Terminal('2')], [Terminal('3')], [Terminal('4')], [Terminal('5')], [Terminal('6')]] }
Some(ContextFree)
```
