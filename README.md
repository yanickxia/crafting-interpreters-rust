# crafting interpreters

impl [crafting interpreters](http://www.craftinginterpreters.com/) by Rust

## TODO

- [resolver](http://www.craftinginterpreters.com/resolving-and-binding.html) 未涉及，这部分因为要求互相引用，Env 更像一个
  LinkedList，这部分先不涉及了。

```text
var a = "global";
{
  fun showA() {
    print a;
  }

  showA();
  var a = "block";
  showA();
}
```

这部分无法正常工作，打印是 `global` `global`