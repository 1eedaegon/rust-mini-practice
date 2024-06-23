// use core::fmt;
use std::{error::Error, fmt::{self}};

use axum::{http::{Response, StatusCode}, response::IntoResponse};
use chrono::DateTime;
use chrono::Utc;

/*
    =====================
    =====================
    ==== 1. Traits ======
    =====================
    =====================

*/

// 러스트의 trait은 scala의 trait과 다르게 다중 상속을 통한 mixin이 불가하다.
// 대신, '+'연산으로 trait multiple bound를 통해 다중 상속을 흉내낼 수 있다.
// 다만, rust의 trait은 문법에서 upper/lower/context bound를 제공하지 않는다.

/*
아래는 내가 trait을 활용해서 spark에 사용한 mixin 이다.

case class dataExample(name: String, age: Int)
trait CustomSerializable extends Serializable

object SparkAppWithSerializable extends App with CustomSerializable {
  val spark = SparkSession.builder
    .appName("SparkAppWithSerializable")
    .master("local[*]")
    .getOrCreate()

  val sc = spark.sparkContext

  val data = Seq(Person("Alice", 28), Person("Bob", 32), Person("Catherine", 25))
  val rdd = sc.parallelize(data)

  val adults = rdd.filter(_.age >= 30).collect()

  adults.foreach(println)

  spark.stop() 


위 scala코드에서 extends App with CustomSerializable은 trait bound와 mixin을 동시에 사용한 예제이다.
upper bound와 혼합하면 아래와 같은 형태가 나온다.

class Zoo[T <: Animal with HasLegs](val animal: T) {
  def showInfo(): Unit = {
    println(s"Animal: ${animal.name}, Legs: ${animal.numberOfLegs}")
  }
}

rust에서는 upper bound나 trait/mixin 동시 사용은 안되지만, generic을 활용한 multiple bound를 할 수 있다. 아래에서 보자.

use std::fmt::Debug;
use std::fmt::Display;

fn print_info<T: Display + Debug>(item: T) {
    println!("Display: {}", item);
    println!("Debug: {:?}", item);
}

fn main() {
    let value = 42;
    print_info(value);
}

위의 fn print_into<T: Display + Debug>(item: T){}는 Display와 Debug trait을 동시에 사용하는 예제이다.
T에 대한 구현이 Display도 가능하고 Debug도 가능하다는 의미다.
헷갈리면 안되는 것이 위의 구현은 generic을 이용한 타입연산이기 때문에 pointer를 이용한 dynamic dispatch가 아닌 static dispatch이다.

당연히 Golang의 pointer reciever처럼 runtime 참조 또한 가능하다.
vtable에 포인터를 저장하고  dynamic dispatch를 통한 runtime method calling이 가능하다.
즉, 다형성을 위한 syntax가 존재하는데, clang처럼 포인터를 저장하고 호출하거나
golang처럼 "func(*structName) func(){}" 식의 포인터 호출이 아닌 "impl ~ for ~로 호출이 가능하다."

example: impl traitName for structName{}
다만, GC를 통한 메모리수집이나 다중 상속을 허용하지 않으므로 trait bound와 mixin을 동시에 사용하는 것은 불가능하다.

후... Rust의 trait파트는 너무 쓸말이 많으니 넘어가자.
*/

/*
    ====================
    ====================
    ==== 2. Enums ======
    ====================
    ====================
*/

// Rust의 enum은 언어에서 지원하는 강력한 타입이다.
enum Number {
    Odd(i64),
    Even(i64),
}

impl Number {
    // 아래 처럼 if로 해도 되지만, match를 사용해서 깔끔하게 풀 수 있다.

    // fn from_i64(num: i64) -> Self {
    //     if num % 2 == 0 {
    //         Number::Even(num),
    //     } else {
    //         Number::Odd(num),
    //     }
    // }

    fn from_i64(num: i64) -> Self {
        match num % 2 == 0 {
            true => Number::Even(num),
            false => Number::Odd(num),
        }
    }

    // match num % 2 {
    //     0 -> Number::Even(num as i32),
    //     _ -> Number::Odd(num as i32),
    // }
    // *ns optimization
}

//  Enum as error types
/*
    Rust의 enum은 언어의 지원을 받는다.
    golang의 const와 iota로 syntatic하게 사용하는 것과는 다르게 enum이라는 타입을 지정해서 사용할 수 있다.
    아래는 내가 cloud image driver에 사용했던 Golang의 예제이다.

    /// errors.go
    type DriverError interface {
        error
        GetErrLevel() ErrorLevel
    }
    type DriverType int

    const (
        UnknownDriverType DriverType = iota
        ...
    )
    const (
        apiDriverType                = 100
        HttpApiDriverType DriverType = iota + apiDriverType
        ...
    )

    func (t DriverType) String() string {
        switch t {
        case HttpApiDriverType:
            return "HttpApiDriverType"
        case GrpcGwDriverType:
            return "GrpcGwDriverType"
        case KubeletClientDriverType:
            return "KubeletClientDriverType"
        case AwsClientDriverType:
            return "AwsClientDriverType"
        default:
            return "UnknownDriverType"
        }
    }

    iota라는 incremental한 선언으로 사용하는 것으로 enum을 지정한다.

    Rust에서는 enum을 사용해서 아래와 같이 사용할 수 있다.
    한번 보자.
*/

// derive(debug)매크로 선언을 통해 Debugging을 위한 출력을 사용할 수 있다.
#[derive(Debug)]
enum MyError {
    SQLError(sqlx::Error),
    RedisError(redis::RedisError),
    Forbidden,
    NotFound,
    Unauthorized,
}

impl fmt::Display for MyError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            MyError::SQLError(e) => write!(f, "SQL Error: {e}"),
            // MyError::SQLError(e) => write!(f, format!("SQL Error: {e}")),
            MyError::RedisError(e) => write!(f, "Redis Error: {e}"),
            MyError::Forbidden => write!(f, "Forbidden"),
            MyError::NotFound => write!(f, "Not Found"),
            MyError::Unauthorized => write!(f, "Unauthorized"),
        }
    }
}
/*
    각 에러에 대해 switch-case를 사용했던 것과 같이 match를 사용해서 처리한다.
    특히 Debug 매크로를 사용하면 별도의 출력을 위한 코드를 작성하지않아도된다.
    하지만 Debug 매크로를 사용하려면, 강타입의 trait 구현을 위해 string변환하는 fmt::Display를 구현해주어야한다.
    rust에선 derive(Debug)의 구현은 fmt::Display를 강제한다.
    fmt::Display를 구현하면, println!("{}", MyError::Forbidden)과 같이 사용할 수 있다.

    * format!: 포맷된 문자열을 사용하는 매크로. js의 `hello {$world}`와 같은 템플릿 리터럴과 유사하다.
    * write!: 출력 대상이 std::fmt::write를 구현한 버퍼나 파일같은 대상에 쓰기를 수행하는 매크로.
    * fmt::Formatter: 출력을 위한 구조체, {}나 {:?}와 같은 String placeholder를 사용할 수 있다.
    * fmt::Result: write!나 format!의 결과를 반환한다. Result를 반환하는 이유는 Rust의 에러처리 방식 때문임

    Error trait을 자세히 보면 아래와 같음.

    pub trait Error: Debug + Display {
        fn description(&self) -> &str { ... }
        fn source(&self) -> Option<&(dyn Error + 'static)> { ...}
    }
    모두 구현해야하나? 아님, Optional하기 때문에 필요한 것만 구현하면 된다.
*/

// 아래 trait impl로 Error trait을 구현한다.
impl Error for MyError{}

// 다른 web app은 어떻게 했을까?
// Axum
impl IntoResponse for MyError {
    fn into_response(&self) -> Response {
        match self {
            MyError::SQLError(e) => (StatusCode::INTERNAL_SERVER_ERROR, {"SQL Error {e}"}),
            MyError::RedisError(e) => (StatusCode::INTERNAL_SERVER_ERROR, {"REDIS Error {e}"}),
            MyError::Forbidden => (StatusCode::FORBIDDEN, "Forbidden".to_string()).into_response(),
            MyError::NotFound => (StatusCode::NOT_FOUND, "Not Found".to_string()).into_response(),
            MyError::Unauthorized => (StatusCode::UNAUTHORIZED, "Unauthorized".to_string()).into_response(),
        }
    }
}

// Enum as wrapper types
/*
    언어에서 기본 제공하는 Enum을 좀 더 누려보자.
    password에 secured옵션과 unsecured 옵션을 나눈다.
*/

struct Password {
    password: String,
    created_at: DateTime<Utc>,
}

enum PasswordEnum {
    Secured(Password),
    Unsecured(Password),
}
// enum에 trait을 붙여서 출력이 가능하게 해보자.
// 아래 코드는 메서드를 
// Dynamic dispatch
impl fmt::Display for PasswordEnum {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Formatter {
        match self {
            PasswordEnum::Secured(password) => {
                password = password.chars().map(|_| '*'.to_owned()).collect::<String>();
                write!(f, p);
            },
            PasswordEnum::Unsecured(p) => {
                p = p.chars().map(|_| '*'.to_owned())
            }
        }
    }
}

impl PasswordEnum {
    fn is_secured(&self) -> bool {
        match self {
            PasswordEnum::Secured(_) => true,
            PasswordEnum::Unsecured(_) => false,
        }
    }
}

/*
    ====================
    ====================
    ==== 3. Macros =====
    ====================
    ====================

    

*/



/*
    =====================
    =====================
    ==== 4. Patterns ====
    =====================
    =====================
*/

/*
    references:
        - my head with
        - cloudflare
        - shuttle
*/
fn main() {
    println!("Hello, world!");
}
