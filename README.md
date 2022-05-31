`[acc_attributes]`

1. mut  
account.is_writable()
2. has_one  
acount.authority(field in accoun state) === authority.key()
3. payer = {acc_in_struct}, must be with `init`
build the PDA 
4. bump   
if empty --> no need of verfication, create by canonical bumo   
is assigned the value --> check the verfication by executing find_program_address
    



questions:

1. sigenr !== authority 
2. how to fix unchecked situation with accounts of `authorty & realm`
